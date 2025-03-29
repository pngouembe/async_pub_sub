use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

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
        panic!("Struct must have at least one field that implements the Subscriber trait or is marked with #[subscriber]");
    }

    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = if subscriber_fields.len() == 1 {
        // Single publisher case - implement Publisher trait
        let (field, message_type) = subscriber_fields.first().expect("subscriber_fields is not empty, this should never happen");
        let field_name = &field.ident;

        quote! {
            impl #impl_generics async_pub_sub::Subscriber for #struct_name #ty_generics #where_clause {
                type Message = #message_type;

                fn get_name(&self) -> &'static str {
                    async_pub_sub::Subscriber::get_name(&self.#field_name)
                }

                fn subscribe_to(&mut self, publisher: &mut impl async_pub_sub::MultiPublisher<Self::Message>) -> async_pub_sub::Result<()> {
                    async_pub_sub::Subscriber::subscribe_to(&mut self.#field_name, publisher)
                }

                fn receive(&mut self) -> impl std::future::Future<Output = Self::Message> {
                    async_pub_sub::Subscriber::receive(&mut self.#field_name)
                }
            }
        }
    } else {
        // Multiple publishers case - implement MultiPublisher trait for each message type
        let impls = subscriber_fields.iter().map(|(field, message_type)| {
            let field_name = &field.ident;

            quote! {
                impl #impl_generics async_pub_sub::MultiSubscriber<#message_type> 
                    for #struct_name #ty_generics #where_clause 
                {
                    fn get_subscriber(&self) -> &impl async_pub_sub::Subscriber<Message = #message_type> {
                        &self.#field_name
                    }

                    fn get_subscriber_mut(&mut self) -> &mut impl async_pub_sub::Subscriber<Message = #message_type> {
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
