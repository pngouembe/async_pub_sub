use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

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
        panic!("Struct must have at least one field that implements the Publisher trait or is marked with #[publisher]");
    }

    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();


    let expanded = if publisher_fields.len() == 1 {
        // Single publisher case - implement Publisher trait
        let (field, message_type) = publisher_fields.first().unwrap();
        let field_name = &field.ident;

        quote! {
            impl #impl_generics tokio_pub_sub::Publisher for #struct_name #ty_generics #where_clause {
                type Message = #message_type;

                fn get_name(&self) -> &'static str {
                    tokio_pub_sub::Publisher::get_name(&self.#field_name)
                }

                fn publish(&self, message: Self::Message) -> futures::future::BoxFuture<tokio_pub_sub::Result<()>> {
                    tokio_pub_sub::Publisher::publish(&self.#field_name, message)
                }

                fn get_message_stream(
                    &mut self,
                    subscriber_name: &'static str,
                ) -> tokio_pub_sub::Result<std::pin::Pin<Box<dyn futures::Stream<Item = Self::Message> + Send + Sync + 'static>>> {
                    tokio_pub_sub::Publisher::get_message_stream(&mut self.#field_name, subscriber_name)
                }
            }
        }
    } else {
        // Multiple publishers case - implement MultiPublisher trait for each message type
        let impls = publisher_fields.iter().map(|(field, message_type)| {
            let field_name = &field.ident;

            quote! {
                impl #impl_generics tokio_pub_sub::MultiPublisher<#message_type> 
                    for #struct_name #ty_generics #where_clause 
                {
                    fn get_publisher(&self) -> &impl tokio_pub_sub::Publisher<Message = #message_type> {
                        &self.#field_name
                    }

                    fn get_publisher_mut(&mut self) -> &mut impl tokio_pub_sub::Publisher<Message = #message_type> {
                        &mut self.#field_name
                    }
                }
            }
        });

        quote! {
            #(#impls)*
        }
    };

    TokenStream::from(expanded)
}
