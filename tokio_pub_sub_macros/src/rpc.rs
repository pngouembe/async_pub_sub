use heck::ToUpperCamelCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ImplItemFn, Item};

pub(crate) fn generate_rpc_interface(input: Item) -> TokenStream {
    let input = match input {
        Item::Impl(input) => input,
        _ => panic!("The rpc_interface macro can only be used on impl blocks"),
    };

    let trait_name = input
        .trait_
        .as_ref()
        .expect("Must be a trait implementation")
        .1
        .segments
        .last()
        .expect("Trait path must not be empty")
        .ident
        .clone();

    let struct_name = if let syn::Type::Path(type_path) = *input.self_ty.clone() {
        type_path.path.segments.last().unwrap().ident.clone()
    } else {
        panic!("Expected a path for the self type")
    };

    let methods: Vec<_> = input
        .items
        .iter()
        .filter_map(|item| {
            if let syn::ImplItem::Fn(method) = item {
                Some(method)
            } else {
                None
            }
        })
        .collect();

    let enum_name = format_ident!("{}Functions", trait_name);
    let enum_variants = generate_enum_variants(&methods);
    let run_impl = generate_run_implementation(&enum_name, &struct_name, &methods);

    let expanded = quote! {
        #input

        pub enum #enum_name {
            #(#enum_variants)*
        }

        #run_impl
    };

    expanded.into()
}

fn generate_enum_variants<'a>(
    methods: &'a [&'a ImplItemFn],
) -> impl Iterator<Item = proc_macro2::TokenStream> + 'a {
    methods.iter().map(|method| {
        let name = &method.sig.ident;
        let variant_name = format_ident!("{}", name.to_string().to_upper_camel_case());

        let input_type = if let syn::FnArg::Typed(pat_type) = &method.sig.inputs[1] {
            &pat_type.ty
        } else {
            panic!("Expected a typed argument for the method input")
        };

        let output_type = if let syn::ReturnType::Type(_, ty) = &method.sig.output {
            ty
        } else {
            panic!("Expected a return type for the method output")
        };

        quote! {
            #variant_name(tokio_pub_sub::Request<#input_type, #output_type>),
        }
    })
}

fn generate_run_implementation(
    enum_name: &syn::Ident,
    struct_name: &syn::Ident,
    methods: &[&ImplItemFn],
) -> proc_macro2::TokenStream {
    let match_arms = methods.iter().map(|method| {
        let name = &method.sig.ident;
        let variant_name = format_ident!("{}", name.to_string().to_upper_camel_case());
        quote! {
            #enum_name::#variant_name(req) => {
                let tokio_pub_sub::Request {
                    content,
                    response_sender,
                } = req;

                let response = self.#name(content).await;
                let _ = response_sender.send(response);
            }
        }
    });

    quote! {
        impl<S> #struct_name<S>
        where
            S: Subscriber<Message = #enum_name>,
        {
            pub async fn run(&mut self) {
                loop {
                    let request = self.subscriber.receive().await;
                    match request {
                        #(#match_arms)*
                    }
                }
            }
        }
    }
}
