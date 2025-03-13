use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, GenericParam, Path, Type, TypePath};

#[proc_macro_derive(DeriveSubscriber)]
pub fn derive_subscriber(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // Get the fields
    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields_named) => &fields_named.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    // Find the field that implements Subscriber
    let subscriber_field = fields
        .iter()
        .find(|field| {
            if let Type::Path(TypePath {
                path: Path { segments, .. },
                ..
            }) = &field.ty
            {
                if let Some(generic_param) = input.generics.params.iter().find(|p| {
                    if let GenericParam::Type(type_param) = p {
                        segments.iter().any(|s| s.ident == type_param.ident)
                    } else {
                        false
                    }
                }) {
                    if let GenericParam::Type(type_param) = generic_param {
                        return type_param.bounds.iter().any(|bound| {
                            if let syn::TypeParamBound::Trait(trait_bound) = bound {
                                trait_bound
                                    .path
                                    .segments
                                    .last()
                                    .map(|s| s.ident == "Subscriber")
                                    .unwrap_or(false)
                            } else {
                                false
                            }
                        });
                    }
                }
            }
            false
        })
        .expect("Struct must have a field that implements the Subscriber trait");

    let field_name = &subscriber_field.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics tokio_pub_sub::Subscriber for #struct_name #ty_generics #where_clause {
            type Message = <S as tokio_pub_sub::Subscriber>::Message;

            fn get_name(&self) -> &'static str {
                self.#field_name.get_name()
            }

            fn subscribe_to(
                &mut self,
                publisher: &mut impl tokio_pub_sub::Publisher<Message = Self::Message>,
            ) -> tokio_pub_sub::Result<()> {
                self.#field_name.subscribe_to(publisher)
            }

            fn receive(&mut self) -> impl std::future::Future<Output = Self::Message> {
                self.#field_name.receive()
            }
        }
    };

    TokenStream::from(expanded)
}
