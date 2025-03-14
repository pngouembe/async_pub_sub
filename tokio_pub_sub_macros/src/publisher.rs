use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Type, TypePath};

use crate::helpers::find_publisher_field;

pub(crate) fn derive_publisher_impl(input: DeriveInput) -> TokenStream {
    let struct_name = &input.ident;

    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields_named) => &fields_named.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let publisher_field = find_publisher_field(fields, &input)
        .expect("Struct must have a field that implements the Publisher trait");

    let field_name = &publisher_field.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let type_param = if let Type::Path(TypePath { path, .. }) = &publisher_field.ty {
        path.segments
            .first()
            .map(|s| &s.ident)
            .expect("Invalid field type")
    } else {
        panic!("Invalid field type")
    };

    let expanded = quote! {
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
    };

    TokenStream::from(expanded)
}
