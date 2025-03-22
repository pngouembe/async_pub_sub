use heck::ToUpperCamelCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::Item;

pub(crate) fn generate_rpc_interface(input: Item) -> TokenStream {
    let input = match input {
        Item::Trait(input) => input,
        _ => panic!("The rpc_interface macro can only be used on trait definitions"),
    };

    let trait_name = input.ident.clone();
    let message_enum_name = format_ident!("{}Message", trait_name);
    let client_trait_name = format_ident!("{}Client", trait_name);
    let server_trait_name = format_ident!("{}Server", trait_name);

    let methods: Vec<_> = input
        .items
        .iter()
        .filter_map(|item| {
            if let syn::TraitItem::Fn(method) = item {
                Some(method)
            } else {
                None
            }
        })
        .collect();

    let enum_variants = generate_enum_variants(&methods);
    let client_methods = generate_client_methods(&message_enum_name, &methods);
    let trait_impl_for_client =
        generate_trait_impl_for_client(&trait_name, &client_trait_name, &methods);
    let server_impl = generate_server_impl(&message_enum_name, &trait_name, &methods);
    let server_trait_impl =
        generate_server_trait_impl(&server_trait_name, &message_enum_name, &trait_name);

    let expanded = quote! {
        #input

        #[derive(Debug)]
        pub enum #message_enum_name {
            #(#enum_variants)*
        }

        pub trait #client_trait_name: tokio_pub_sub::MultiPublisher<#message_enum_name> {
            #(#client_methods)*
        }

        #trait_impl_for_client

        pub trait #server_trait_name: tokio_pub_sub::MultiSubscriber<#message_enum_name> + #trait_name {
            async fn run(&mut self) {
                loop {
                    let request = self.receive().await;
                    self.handle_request(request).await;
                }
            }

            async fn handle_request(&mut self, request: #message_enum_name) {
                match request {
                    #(#server_impl)*
                }
            }
        }

        #server_trait_impl
    };

    expanded.into()
}

fn generate_enum_variants<'a>(
    methods: &'a [&'a syn::TraitItemFn],
) -> impl Iterator<Item = proc_macro2::TokenStream> + 'a {
    methods.iter().map(|method| {
        let name = &method.sig.ident;
        let variant_name = format_ident!("{}", name.to_string().to_upper_camel_case());

        let input_types: Vec<_> = method
            .sig
            .inputs
            .iter()
            .filter_map(|input| match input {
                syn::FnArg::Typed(pat_type) => Some(&pat_type.ty),
                syn::FnArg::Receiver(_) => None, // ignore self
            })
            .collect();

        let input_types = if input_types.is_empty() {
            quote! { () }
        } else if input_types.len() == 1 {
            let ty = input_types.first().unwrap();
            quote! { #ty }
        } else {
            quote! { (#(#input_types),*) }
        };

        let output_type = match &method.sig.output {
            syn::ReturnType::Type(_, ty) => quote! { #ty },
            syn::ReturnType::Default => quote! { () },
        };

        quote! {
            #variant_name(tokio_pub_sub::Request<#input_types, #output_type>),
        }
    })
}

fn generate_client_methods<'a>(
    message_enum_name: &'a syn::Ident,
    methods: &'a [&'a syn::TraitItemFn],
) -> impl Iterator<Item = proc_macro2::TokenStream> + 'a {
    methods.iter().map(move |method| {
        let name = &method.sig.ident;
        let variant_name = format_ident!("{}", name.to_string().to_upper_camel_case());
        let args = &method.sig.inputs;
        let output = &method.sig.output;

        let function_signature = quote! { #name(#args) #output };

        let request_content: Vec<_> = args
            .iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Receiver(_) => None,
                syn::FnArg::Typed(pat_ty) => Some(&pat_ty.pat),
            })
            .collect();

        let request_content = if request_content.is_empty() {
            quote! { () }
        } else if request_content.len() == 1 {
            let arg_name = request_content.first().unwrap();
            quote! { #arg_name }
        } else {
            quote! { (#(#request_content),*) }
        };

        quote! {
            async fn #function_signature {
                let (request, response) = tokio_pub_sub::Request::new(#request_content);
                self.publish(#message_enum_name::#variant_name(request))
                    .await
                    .unwrap();
                response.await.unwrap()
            }
        }
    })
}

fn generate_trait_impl_for_client(
    trait_name: &syn::Ident,
    client_trait_name: &syn::Ident,
    methods: &[&syn::TraitItemFn],
) -> proc_macro2::TokenStream {
    let method_impls = methods.iter().map(|method| {
        let name = &method.sig.ident;
        let args = &method.sig.inputs;
        let output = &method.sig.output;

        // Extract argument names excluding &self
        let arg_names: Vec<_> = args
            .iter()
            .skip(1)
            .map(|arg| {
                if let syn::FnArg::Typed(pat_type) = arg {
                    if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                        &pat_ident.ident
                    } else {
                        panic!("Expected identifier pattern for argument")
                    }
                } else {
                    panic!("Expected typed argument")
                }
            })
            .collect();

        let function_signature = quote! { #name(#args) #output };

        quote! {
            async fn #function_signature {
                <Self as #client_trait_name>::#name(self, #(#arg_names),*).await
            }
        }
    });

    quote! {
        impl<T> #trait_name for T
        where
            T: #client_trait_name,
        {
            #(#method_impls)*
        }
    }
}

fn generate_server_impl<'a>(
    message_enum_name: &'a syn::Ident,
    trait_name: &'a syn::Ident,
    methods: &'a [&'a syn::TraitItemFn],
) -> impl Iterator<Item = proc_macro2::TokenStream> + 'a {
    methods.iter().map(move |method| {
        let name = &method.sig.ident;
        let variant_name = format_ident!("{}", name.to_string().to_upper_camel_case());

        let arg_names: Vec<_> = method
            .sig
            .inputs
            .iter()
            .filter_map(|input| match input {
                syn::FnArg::Typed(pat_type) => Some(&pat_type.pat),
                syn::FnArg::Receiver(_) => None, // ignore self
            })
            .collect();

        let function_call = if arg_names.is_empty() {
            quote! { let response = <Self as #trait_name>::#name(self).await; }
        } else if arg_names.len() == 1 {
            quote! { let response = <Self as #trait_name>::#name(self, content).await; }
        } else {
            quote! {
                let (#(#arg_names),*) = content;
                let response = <Self as #trait_name>::#name(self, #(#arg_names),*).await;
            }
        };

        quote! {
            #message_enum_name::#variant_name(req) => {
                let tokio_pub_sub::Request {
                    content,
                    response_sender,
                } = req;
                #function_call
                let _ = response_sender.send(response);
            }
        }
    })
}

fn generate_server_trait_impl(
    server_trait_name: &syn::Ident,
    message_enum_name: &syn::Ident,
    trait_name: &syn::Ident,
) -> proc_macro2::TokenStream {
    quote! {
        impl<T> #server_trait_name for T where
            T: #trait_name + tokio_pub_sub::MultiSubscriber<#message_enum_name>
        {
        }
    }
}
