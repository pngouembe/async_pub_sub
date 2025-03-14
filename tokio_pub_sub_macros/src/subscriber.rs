use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Type, TypePath};

use crate::helpers::find_subscriber_field;

pub(crate) fn derive_subscriber_impl(input: DeriveInput) -> TokenStream {
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
    let subscriber_field = find_subscriber_field(fields, &input)
        .expect("Struct must have a field that implements the Subscriber trait");

    let field_name = &subscriber_field.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Get generic type parameter name from field type
    let type_param = if let Type::Path(TypePath { path, .. }) = &subscriber_field.ty {
        path.segments
            .first()
            .map(|s| &s.ident)
            .expect("Invalid field type")
    } else {
        panic!("Invalid field type")
    };

    let expanded = quote! {
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
    };

    TokenStream::from(expanded)
}
