use std::{collections::HashMap, fmt::Debug};

use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::helpers::{find_pub_sub_types_in_generics, message_type_from_path_opt};

pub(crate) fn derive_subscriber_impl(input: DeriveInput) -> TokenStream {
    InputStruct::try_from(input)
        .and_then(|input| Ok(input.generate_code()))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

struct InputStruct {
    context: CodeGenerationContext,
    subscriber_fields: Vec<SubscriberField>,
    is_multi_subscriber: bool,
}

impl TryFrom<DeriveInput> for InputStruct {
    type Error = syn::Error;

    fn try_from(input: DeriveInput) -> Result<Self, Self::Error> {
        let context = CodeGenerationContext {
            struct_name: input.ident.clone(),
            generics: input.generics.clone(),
        };

        let subscriber_fields = find_all_subscribers(&input)?;

        if subscriber_fields.is_empty() {
            return Err(syn::Error::new_spanned(
                input,
                "DeriveSubscriber macro requires that your struct must have at least one field that implements the Subscriber trait or is marked with #[subscriber]",
            ));
        }

        let is_multi_subscriber = subscriber_fields.len() > 1;
        Ok(Self {
            context,
            subscriber_fields,
            is_multi_subscriber,
        })
    }
}

impl InputStruct {
    fn generate_code(&self) -> proc_macro2::TokenStream {
        if self.is_multi_subscriber {
            self.subscriber_fields
                .iter()
                .fold(quote! {}, |generated_code, subscriber_field| {
                    let impl_code = subscriber_field.generate_wrapper_impl(&self.context);
                    quote! {
                        #generated_code
                        #impl_code
                    }
                })
        } else {
            let subscriber_field = self
                .subscriber_fields
                .first()
                .expect("subscriber_fields is not empty, this should never happen");

            subscriber_field.generate_impl(&self.context)
        }
    }
}

struct CodeGenerationContext {
    struct_name: syn::Ident,
    generics: syn::Generics,
}

struct SubscriberField {
    name: proc_macro2::TokenStream,
    message_type: proc_macro2::TokenStream,
}

impl Debug for SubscriberField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubscriberField")
            .field("name", &self.name.to_string())
            .field("message_type", &self.message_type.to_string())
            .finish()
    }
}

impl SubscriberField {
    fn new(name: proc_macro2::TokenStream, message_type: proc_macro2::TokenStream) -> Self {
        Self { name, message_type }
    }

    fn from_field_attributes_opt(field: &syn::Field) -> Option<Self> {
        let message_type = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("subscriber"))
            .and_then(|attr| {
                let ty = attr.parse_args::<syn::Type>().ok()?;
                Some(quote! { #ty })
            })?;

        let field_name = field.ident.clone().map(|ident| quote! { #ident })?;

        Some(Self::new(field_name, message_type))
    }

    fn from_field_type_opt(
        field: &syn::Field,
        generic_subscribers: &HashMap<syn::Ident, proc_macro2::TokenStream>,
    ) -> Option<Self> {
        let syn::Type::Path(syn::TypePath { path, .. }) = &field.ty else {
            return None;
        };

        let Some(ident) = path.get_ident() else {
            return None;
        };

        let message_type = generic_subscribers
            .get(ident)
            .cloned()
            .or_else(|| message_type_from_path_opt(path, "Subscriber"))?;

        let field_name = field.ident.clone().map(|ident| quote! { #ident })?;

        Some(Self::new(field_name, message_type))
    }
}

impl SubscriberField {
    fn generate_impl(&self, context: &CodeGenerationContext) -> proc_macro2::TokenStream {
        let field_name = &self.name;
        let message_type = &self.message_type;
        let struct_name = &context.struct_name;
        let (impl_generics, ty_generics, where_clause) = &context.generics.split_for_impl();

        quote! {
            impl #impl_generics async_pub_sub::Subscriber for #struct_name #ty_generics #where_clause {
                type Message = #message_type;

                fn get_name(&self) -> &'static str {
                    async_pub_sub::Subscriber::get_name(&self.#field_name)
                }

                fn subscribe_to(&mut self, publisher: &mut impl async_pub_sub::PublisherWrapper<Self::Message>) -> async_pub_sub::Result<()> {
                    async_pub_sub::Subscriber::subscribe_to(&mut self.#field_name, publisher)
                }

                fn receive(&mut self) -> impl std::future::Future<Output = Self::Message> {
                    async_pub_sub::Subscriber::receive(&mut self.#field_name)
                }
            }
        }
    }

    fn generate_wrapper_impl(&self, context: &CodeGenerationContext) -> proc_macro2::TokenStream {
        let field_name = &self.name;
        let message_type = &self.message_type;
        let struct_name = &context.struct_name;
        let (impl_generics, ty_generics, where_clause) = &context.generics.split_for_impl();

        quote! {
            impl #impl_generics async_pub_sub::SubscriberWrapper<#message_type>
            for #struct_name #ty_generics #where_clause {
                fn get_subscriber(&self) -> &impl async_pub_sub::Subscriber<Message = #message_type> {
                    &self.#field_name
                }

                fn get_subscriber_mut(&mut self) -> &mut impl async_pub_sub::Subscriber<Message = #message_type> {
                    &mut self.#field_name
                }
            }
        }
    }
}

fn find_all_subscribers(input: &DeriveInput) -> Result<Vec<SubscriberField>, syn::Error> {
    let generic_subscribers = find_pub_sub_types_in_generics("Subscriber", &input.generics);

    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields_named) => &fields_named.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    &input,
                    "DeriveSubscriber macro only supports structs with named fields",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "DeriveSubscriber macro only supports structs",
            ))
        }
    };

    let subscriber_fields = fields
        .iter()
        .filter_map(|field| {
            SubscriberField::from_field_attributes_opt(field).or(
                SubscriberField::from_field_type_opt(field, &generic_subscribers),
            )
        })
        .collect();

    Ok(subscriber_fields)
}
