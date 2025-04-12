use proc_macro::TokenStream;
use quote::quote;
use std::{collections::HashMap, fmt::Debug};
use syn::DeriveInput;

use crate::helpers::{find_pub_sub_types_in_generics, message_type_from_path_opt};

pub(crate) fn derive_publisher_impl(input: DeriveInput) -> TokenStream {
    InputStruct::try_from(input)
        .and_then(|input| Ok(input.generate_code()))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

struct InputStruct {
    context: CodeGenerationContext,
    publisher_fields: Vec<PublisherField>,
    is_multi_publisher: bool,
}

impl TryFrom<DeriveInput> for InputStruct {
    type Error = syn::Error;

    fn try_from(input: DeriveInput) -> Result<Self, Self::Error> {
        let context = CodeGenerationContext {
            struct_name: input.ident.clone(),
            generics: input.generics.clone(),
        };

        let publisher_fields = find_all_publishers(&input)?;

        if publisher_fields.is_empty() {
            return Err(syn::Error::new_spanned(
                input,
                "DerivePublisher macro requires that your struct must have at least one field that implements the Publisher trait or is marked with #[publisher]",
            ));
        }

        let is_multi_publisher = publisher_fields.len() > 1;
        Ok(Self {
            context,
            publisher_fields,
            is_multi_publisher,
        })
    }
}

impl InputStruct {
    fn generate_code(&self) -> proc_macro2::TokenStream {
        if self.is_multi_publisher {
            self.publisher_fields
                .iter()
                .fold(quote! {}, |generated_code, publisher_field| {
                    let impl_code = publisher_field.generate_wrapper_impl(&self.context);
                    quote! {
                        #generated_code
                        #impl_code
                    }
                })
        } else {
            let publisher_field = self
                .publisher_fields
                .first()
                .expect("publisher_fields is not empty, this should never happen");

            publisher_field.generate_impl(&self.context)
        }
    }
}

struct CodeGenerationContext {
    struct_name: syn::Ident,
    generics: syn::Generics,
}

struct PublisherField {
    name: proc_macro2::TokenStream,
    message_type: proc_macro2::TokenStream,
}

impl Debug for PublisherField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PublisherField")
            .field("name", &self.name.to_string())
            .field("message_type", &self.message_type.to_string())
            .finish()
    }
}

impl PublisherField {
    fn new(name: proc_macro2::TokenStream, message_type: proc_macro2::TokenStream) -> Self {
        Self { name, message_type }
    }

    fn from_field_attributes_opt(field: &syn::Field) -> Option<Self> {
        let message_type = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("publisher"))
            .and_then(|attr| {
                let ty = attr.parse_args::<syn::Type>().ok()?;
                Some(quote! { #ty })
            })?;

        let field_name = field.ident.clone().map(|ident| quote! { #ident })?;

        Some(Self::new(field_name, message_type))
    }

    fn from_field_type_opt(
        field: &syn::Field,
        generic_publishers: &HashMap<syn::Ident, proc_macro2::TokenStream>,
    ) -> Option<Self> {
        let syn::Type::Path(syn::TypePath { path, .. }) = &field.ty else {
            return None;
        };

        let Some(ident) = path.get_ident() else {
            return None;
        };

        let message_type = generic_publishers
            .get(ident)
            .cloned()
            .or_else(|| message_type_from_path_opt(path, "Publisher"))?;

        let field_name = field.ident.clone().map(|ident| quote! { #ident })?;

        Some(Self::new(field_name, message_type))
    }

    fn generate_impl(&self, context: &CodeGenerationContext) -> proc_macro2::TokenStream {
        let field_name = &self.name;
        let message_type = &self.message_type;
        let struct_name = &context.struct_name;
        let (impl_generics, ty_generics, where_clause) = &context.generics.split_for_impl();

        quote! {
            impl #impl_generics async_pub_sub::Publisher for #struct_name #ty_generics #where_clause {
                type Message = #message_type;

                fn get_name(&self) -> &'static str {
                    async_pub_sub::Publisher::get_name(&self.#field_name)
                }

                fn publish(&self, message: Self::Message) -> async_pub_sub::futures::future::BoxFuture<async_pub_sub::Result<()>> {
                    async_pub_sub::Publisher::publish(&self.#field_name, message)
                }

                fn get_message_stream(
                    &mut self,
                    subscriber_name: &'static str,
                ) -> async_pub_sub::Result<std::pin::Pin<Box<dyn async_pub_sub::futures::Stream<Item = Self::Message> + Send + Sync + 'static>>> {
                    async_pub_sub::Publisher::get_message_stream(&mut self.#field_name, subscriber_name)
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
            impl #impl_generics async_pub_sub::PublisherWrapper<#message_type>
            for #struct_name #ty_generics #where_clause {
                fn get_publisher(&self) -> &impl async_pub_sub::Publisher<Message = #message_type> {
                    &self.#field_name
                }

                fn get_publisher_mut(&mut self) -> &mut impl async_pub_sub::Publisher<Message = #message_type> {
                    &mut self.#field_name
                }
            }
        }
    }
}

fn find_all_publishers(input: &DeriveInput) -> Result<Vec<PublisherField>, syn::Error> {
    let generic_publishers = find_pub_sub_types_in_generics("Publisher", &input.generics);

    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields_named) => &fields_named.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    &input,
                    "DerivePublisher macro only supports structs with named fields",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "DerivePublisher macro only supports structs",
            ))
        }
    };

    let publisher_fields = fields
        .iter()
        .filter_map(|field| {
            PublisherField::from_field_attributes_opt(field).or(
                PublisherField::from_field_type_opt(field, &generic_publishers),
            )
        })
        .collect();

    Ok(publisher_fields)
}
