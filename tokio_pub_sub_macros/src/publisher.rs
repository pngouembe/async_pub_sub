use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Type, TypePath};

use crate::helpers::find_all_publisher_fields;

pub(crate) fn derive_publisher_impl(input: DeriveInput) -> TokenStream {
    let struct_name = &input.ident;

    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields_named) => &fields_named.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let publisher_fields = find_all_publisher_fields(fields, &input);
    if publisher_fields.is_empty() {
        panic!("Struct must have at least one field that implements the Publisher trait");
    }

    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let impls = publisher_fields.iter().map(|field| {
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
            impl #impl_generics tokio_pub_sub::Publisher for #struct_name #ty_generics #where_clause {
                type Message = <#type_param as tokio_pub_sub::Publisher>::Message;

                fn get_name(&self) -> &'static str {
                    self.#field_name.get_name()
                }

                fn publish_event(&self, message: Self::Message) -> futures::future::BoxFuture<tokio_pub_sub::Result<()>> {
                    self.#field_name.publish_event(message)
                }

                fn get_message_stream(
                    &mut self,
                    subscriber_name: &'static str,
                ) -> tokio_pub_sub::Result<std::pin::Pin<Box<dyn futures::Stream<Item = Self::Message> + Send + Sync + 'static>>> {
                    self.#field_name.get_message_stream(subscriber_name)
                }
            }
        }
    });

    let expanded = quote! {
        #(#impls)*
    };

    TokenStream::from(expanded)
}
