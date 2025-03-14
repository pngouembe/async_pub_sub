use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Type, TypePath};

use crate::helpers::find_all_subscriber_fields;

pub(crate) fn derive_subscriber_impl(input: DeriveInput) -> TokenStream {
    let struct_name = &input.ident;

    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields_named) => &fields_named.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let subscriber_fields = find_all_subscriber_fields(fields, &input);
    if subscriber_fields.is_empty() {
        panic!("Struct must have at least one field that implements the Subscriber trait");
    }

    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let impls = subscriber_fields.iter().map(|field| {
        let field_name = &field.ident;
        let type_param = if let Type::Path(TypePath { path, .. }) = &field.ty {
            path.segments
                .first()
                .map(|s| &s.ident)
                .expect("Invalid field type")
        } else {
            panic!("Invalid field type")
        };

        quote! {
            impl #impl_generics tokio_pub_sub::Subscriber for #struct_name #ty_generics #where_clause {
                type Message = <#type_param as tokio_pub_sub::Subscriber>::Message;

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
        }
    });

    let expanded = quote! {
        #(#impls)*
    };

    TokenStream::from(expanded)
}
