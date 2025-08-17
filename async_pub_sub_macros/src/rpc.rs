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

    // Extract derives from parsed attributes and ensure Debug is included
    let mut derive_list: Vec<syn::Ident> = attrs.derives.into_iter().collect();

    // Ensure Debug is always included since RequestImpl requires it
    let has_debug = derive_list.iter().any(|ident| *ident == "Debug");
    if !has_debug {
        derive_list.push(syn::Ident::new("Debug", proc_macro2::Span::call_site()));
    }

    let derives: Vec<_> = derive_list.iter().collect();

    let trait_name = input_trait.ident.clone();
    let message_enum_name = format_ident!("{}Message", trait_name);
    let response_enum_name = format_ident!("{}Response", trait_name);
    let content_enum_name = format_ident!("{}Content", trait_name);
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

    let _enum_variants = generate_enum_variants(&methods); // No longer used since we use type alias
    let response_variants = generate_response_variants(&methods);
    let content_variants = generate_content_variants(&methods);
    let request_trait_impl = generate_request_trait_impl(
        &message_enum_name,
        &response_enum_name,
        &content_enum_name,
        &methods,
    );
    let client_methods = generate_client_methods(
        &message_enum_name,
        &response_enum_name,
        &content_enum_name,
        &methods,
    );
    let server_impl = generate_server_impl(
        &message_enum_name,
        &content_enum_name,
        &response_enum_name,
        &trait_name,
        &methods,
    );
    let server_trait_impl = generate_server_trait_impl(
        &server_trait_name,
        &content_enum_name,
        &response_enum_name,
        &trait_name,
    );

    let expanded = quote! {
        #[allow(async_fn_in_trait)]
        #input

        // Generate Content and Response enums with derives
        #[derive(#(#derives),*)]
        pub enum #content_enum_name {
            #(#content_variants)*
        }

        #[derive(#(#derives),*)]
        pub enum #response_enum_name {
            #(#response_variants)*
        }

        // Generate Message as a type alias to RequestImpl
        pub type #message_enum_name = async_pub_sub::RequestImpl<#content_enum_name, #response_enum_name>;

        #request_trait_impl

        pub struct #client_name<OutputMessage = #message_enum_name>
        where
            OutputMessage: Send + 'static,
        {
            pub publisher: Box<dyn async_pub_sub::Publisher<InputMessage = #message_enum_name, OutputMessage = OutputMessage>  + Send>,
        }

        impl<OutputMessage> async_pub_sub::Publisher for #client_name<OutputMessage>
        where
            OutputMessage: Send + 'static,
        {
            type InputMessage = #message_enum_name;
            type OutputMessage = OutputMessage;

            fn get_name(&self) -> &'static str {
                async_pub_sub::Publisher::get_name(&self.publisher)
            }

            fn publish(&self, message: Self::InputMessage) -> async_pub_sub::futures::future::BoxFuture<async_pub_sub::Result<()>> {
                async_pub_sub::Publisher::publish(&self.publisher, message)
            }

            fn get_message_stream(
                &mut self,
                subscriber_name: &'static str,
            ) -> async_pub_sub::Result<std::pin::Pin<Box<dyn async_pub_sub::futures::Stream<Item = Self::OutputMessage> + Send + Sync + 'static>>> {
                async_pub_sub::Publisher::get_message_stream(&mut self.publisher, subscriber_name)
            }
        }

        impl<OutputMessage> #client_name<OutputMessage>
        where
            OutputMessage: Send + 'static,
        {
            pub fn new<P>(publisher: P ) -> Self
            where
                P: async_pub_sub::Publisher<InputMessage = #message_enum_name, OutputMessage = OutputMessage> + Send + 'static,
            {
                Self { publisher: Box::new(publisher) }
            }
        }

        impl<OutputMessage> async_pub_sub::Requester for #client_name<OutputMessage>
        where
            OutputMessage: Send + 'static,
            #message_enum_name: async_pub_sub::Request,
        {
            fn request(
                &self,
                mut request: Self::InputMessage,
            ) -> impl std::future::Future<Output = async_pub_sub::Result<<Self::InputMessage as async_pub_sub::Request>::Response>> {
                use async_pub_sub::Request;
                let response = request.take_response().expect("failed to get response future");
                let publication_future = self.publisher.publish(request);
                async move {
                    publication_future.await?;
                    response.await
                }
            }
        }

        impl<OutputMessage> #trait_name for #client_name<OutputMessage>
        where
            OutputMessage: Send + 'static,
        {
            #(#client_methods)*
        }

        pub trait #server_trait_name<Input, Output>: #trait_name + async_pub_sub::SubscriberWrapper<Input, Output>
        where
            Input: Send + 'static,
            Output: async_pub_sub::Request<Content = #content_enum_name, SentResponse = #response_enum_name> + Send + 'static,
            Output::Response: Send,
        {
            async fn run(&mut self) {
                loop {
                    let request = self.receive().await;
                    self.handle_request(request).await;
                }
            }

            async fn handle_request(&mut self, mut req: Output) {
                use async_pub_sub::Request;
                let content = req.take_content().expect("failed to get content");
                match content {
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
            if let syn::FnArg::Typed(pat_type) = arg
                && let syn::Type::Reference(ty) = &*pat_type.ty {
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

        // Check output for references
        if let syn::ReturnType::Type(_, ty) = &method.sig.output
            && let syn::Type::Reference(ref_ty) = &**ty {
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
            #variant_name(async_pub_sub::RequestImpl<#input_types, #output_type>),
        }
    })
}

fn generate_client_methods<'a>(
    _message_enum_name: &'a syn::Ident,
    response_enum_name: &'a syn::Ident,
    content_enum_name: &'a syn::Ident,
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
            quote! { #name(#args) -> async_pub_sub::futures::future::BoxFuture<'_, #output_type> };

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

        let failure_message = format!("failed to execute {name} request");

        quote! {
            fn #function_signature {
                let content = #content_enum_name::#variant_name(#request_content);
                let request = async_pub_sub::RequestImpl::new(content);

                use async_pub_sub::Requester;
                let response = self.request(request);

                async_pub_sub::futures::FutureExt::boxed(async move {
                    let #response_enum_name::#variant_name(actual_response) = response.await.expect(#failure_message) else {
                        panic!("{}: Expected {} response", #failure_message, stringify!(#variant_name));
                    };
                    actual_response
                })
            }
        }
    })
}

fn generate_server_impl<'a>(
    _message_enum_name: &'a syn::Ident,
    content_enum_name: &'a syn::Ident,
    response_enum_name: &'a syn::Ident,
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
            quote! { let response = <Self as #trait_name>::#name(self, params).await; }
        } else {
            quote! {
                let (#(#arg_names),*) = params;
                let response = <Self as #trait_name>::#name(self, #(#arg_names),*).await;
            }
        };

        quote! {
            #content_enum_name::#variant_name(params) => {
                #function_call
                let response_content = #response_enum_name::#variant_name(response);
                req.respond(response_content).await.expect("failed to send response");
            }
        }
    })
}

fn generate_response_variants<'a>(
    methods: &'a [&'a syn::TraitItemFn],
) -> impl Iterator<Item = proc_macro2::TokenStream> + 'a {
    methods.iter().map(|method| {
        let name = &method.sig.ident;
        let variant_name = format_ident!("{}", name.to_string().to_upper_camel_case());

        let output_type = match &method.sig.output {
            syn::ReturnType::Type(_, ty) => quote! { #ty },
            syn::ReturnType::Default => quote! { () },
        };

        quote! {
            #variant_name(#output_type),
        }
    })
}

fn generate_content_variants<'a>(
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

        quote! {
            #variant_name(#input_types),
        }
    })
}

fn generate_request_trait_impl(
    _message_enum_name: &syn::Ident,
    _response_enum_name: &syn::Ident,
    _content_enum_name: &syn::Ident,
    _methods: &[&syn::TraitItemFn],
) -> proc_macro2::TokenStream {
    // Since PingMessage is now RequestImpl<PingContent, PingResponse>,
    // we don't need to implement Request for it - RequestImpl already does that
    // We just need to provide conversion utilities if needed
    quote! {
        // Helper functions for working with the message type alias could go here if needed
    }
}

fn generate_server_trait_impl(
    server_trait_name: &syn::Ident,
    content_enum_name: &syn::Ident,
    response_enum_name: &syn::Ident,
    trait_name: &syn::Ident,
) -> proc_macro2::TokenStream {
    quote! {
        impl<Input, Output,T> #server_trait_name<Input, Output> for T where
            T: #trait_name + async_pub_sub::SubscriberWrapper<Input, Output>,
            Input: Send + 'static,
            Output: async_pub_sub::Request<Content = #content_enum_name, SentResponse = #response_enum_name> + Send + 'static,
            Output::Response: Send,
        {
        }
    }
}
