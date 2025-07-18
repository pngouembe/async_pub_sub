use heck::ToUpperCamelCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Ident, Item, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

struct AttributeArgs {
    derives: Punctuated<Ident, Token![,]>,
}

impl Parse for AttributeArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(AttributeArgs {
            derives: input.parse_terminated(Ident::parse, Token![,])?,
        })
    }
}

pub(crate) fn generate_rpc_interface(attr: TokenStream, input: Item) -> TokenStream {
   let attrs = parse_macro_input!(attr as AttributeArgs);

    let input_trait = match input.clone() {
        // Clone item for potential error reporting span
        Item::Trait(it) => it,
        _ => {
            return syn::Error::new_spanned(
                input, // Span the whole item passed to the macro
                "The rpc_interface macro can only be used on trait definitions",
            )
            .to_compile_error()
            .into();
        }
    };

    // Extract derives from parsed attributes and convert to iterator
    let derives = attrs.derives.iter();

    let trait_name = input_trait.ident.clone();
    let message_enum_name = format_ident!("{}Message", trait_name);
    let client_name = format_ident!("{}Client", trait_name);
    let server_trait_name = format_ident!("{}Server", trait_name);

    let methods: Vec<_> = input_trait
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

    if let Err(e) = validate_method_signatures(&methods) {
        return e.to_compile_error().into();
    }

    let enum_variants = generate_enum_variants(&methods);
    let client_methods = generate_client_methods(&message_enum_name, &methods);
    let server_impl = generate_server_impl(&message_enum_name, &trait_name, &methods);
    let server_trait_impl =
        generate_server_trait_impl(&server_trait_name, &message_enum_name, &trait_name);

    let expanded = quote! {
        #[allow(async_fn_in_trait)]
        #input

        #[derive(#(#derives),*)]
        pub enum #message_enum_name {
            #(#enum_variants)*
        }

        #[derive(async_pub_sub::macros::DerivePublisher)]
        pub struct #client_name
        {
            #[publisher(#message_enum_name)]
            pub publisher: Box<dyn async_pub_sub::Publisher<Message = #message_enum_name> + Send>,
        }

        impl #client_name
        {
            pub fn new<P>(publisher: P ) -> Self
            where
                P: async_pub_sub::Publisher<Message = #message_enum_name> + Send + 'static,
            {
                Self { publisher: Box::new(publisher) }
            }
        }

        impl #trait_name for #client_name {
            #(#client_methods)*
        }

        pub trait #server_trait_name: async_pub_sub::SubscriberWrapper<#message_enum_name> + #trait_name {
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

fn validate_method_signatures(methods: &[&syn::TraitItemFn]) -> syn::Result<()> {
    // Validate method signatures for references
    for method in methods {
        let method_name = &method.sig.ident;

        // Check inputs for references
        for arg in &method.sig.inputs {
            if let syn::FnArg::Typed(pat_type) = arg {
                if let syn::Type::Reference(ty) = &*pat_type.ty {
                    let arg_name = &pat_type.pat;
                    return Err(syn::Error::new_spanned(
                        &*pat_type.ty,
                        format!(
                            "References in RPC method arguments are not supported yet. Method '{}' uses a reference in its argument '{}' ({}). Please use owned types.",
                            method_name,
                            quote! {#arg_name}, // Attempt to get arg name, might need refinement
                            quote! {#ty}
                        ),
                    ));
                }
            }
        }

        // Check output for references
        if let syn::ReturnType::Type(_, ty) = &method.sig.output {
            if let syn::Type::Reference(ref_ty) = &**ty {
                return Err(syn::Error::new_spanned(
                    &**ty,
                    format!(
                        "References in RPC method return types are not supported yet. Method '{}' returns a reference ({}). Please use owned types.",
                        method_name,
                        quote! {#ref_ty}
                    ),
                ));
            }
        }
    }
    Ok(())
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
            let ty = input_types
                .first()
                .expect("input_types should not be empty");

            quote! { #ty }
        } else {
            quote! { (#(#input_types),*) }
        };

        let output_type = match &method.sig.output {
            syn::ReturnType::Type(_, ty) => quote! { #ty },
            syn::ReturnType::Default => quote! { () },
        };

        quote! {
            #variant_name(async_pub_sub::Request<#input_types, #output_type>),
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
        let output_type = match &method.sig.output {
            syn::ReturnType::Type(_, ty) => quote! { #ty },
            syn::ReturnType::Default => quote! { () },
        };

        let function_signature =
            quote! { #name(#args) -> async_pub_sub::futures::future::BoxFuture<#output_type> };

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
            let arg_name = request_content
                .first()
                .expect("request_content should not be empty");
            quote! { #arg_name }
        } else {
            quote! { (#(#request_content),*) }
        };

        let publish_failure_message = format!("failed to publish {name} request");
        let response_failure_message = format!("failed to receive {name} response");

        quote! {
            fn #function_signature {
                let (request, response) = async_pub_sub::Request::new(#request_content);
                let publish_future = self.publisher.publish(#message_enum_name::#variant_name(request));
                {
                    use async_pub_sub::futures::FutureExt;

                    async move {
                        publish_future
                            .await
                            .expect(#publish_failure_message);
                        response.await.expect(#response_failure_message)
                    }
                    .boxed()
                }
            }
        }
    })
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

        let content = if arg_names.is_empty() {
            quote! { content: _ }
        } else {
            quote! { content }
        };

        quote! {
            #message_enum_name::#variant_name(req) => {
                let async_pub_sub::Request {
                    #content,
                    response_sender,
                } = req;
                #function_call
                response_sender.send(response).expect("failed to send response");
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
            T: #trait_name + async_pub_sub::SubscriberWrapper<#message_enum_name>
        {
        }
    }
}
