use std::{collections::HashMap, fmt::Debug};

use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::helpers::{find_pub_sub_types_in_generics, message_type_from_path_opt};

pub(crate) fn derive_subscriber_impl(input: DeriveInput) -> TokenStream {
    InputStruct::try_from(input)
        .map(|input| input.generate_code())
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
                "DeriveSubscriber macro requires that your struct must have at least one field that implements the Subscriber trait or is marked with #[subscriber].\n\nTip: You can either:\n  1. Add a field with type that implements Subscriber: `field: SomeSubscriber`\n  2. Add #[subscriber] attribute to a field: `#[subscriber(MessageType)] field: SomeType`\n  3. Add #[subscriber] attribute with input/output types: `#[subscriber(InputType, OutputType)] field: SomeType`\n  4. Use generic constraints: `struct MyStruct<S: Subscriber<InputMessage = i32, OutputMessage = String>>`",
            ));
        }

        let is_multi_subscriber = subscriber_fields.len() > 1;

        // Check for conflicting generic subscribers without explicit message type constraints
        if is_multi_subscriber {
            Self::check_for_conflicting_generic_types(&subscriber_fields, &input)?;
        }

        Ok(Self {
            context,
            subscriber_fields,
            is_multi_subscriber,
        })
    }
}

impl InputStruct {
    fn check_for_conflicting_generic_types(
        subscriber_fields: &[SubscriberField],
        input: &DeriveInput,
    ) -> Result<(), syn::Error> {
        // Count how many fields use generic associated types without explicit message type constraints
        let generic_associated_type_count = subscriber_fields
            .iter()
            .filter(|field| {
                // Check if this field uses generic associated types like <T as Subscriber>::InputMessage
                let input_msg_str = field.input_message_type.to_string();
                let output_msg_str = field.output_message_type.to_string();

                // Pattern: <SomeGeneric as async_pub_sub::Subscriber>::InputMessage
                (input_msg_str.contains("as async_pub_sub :: Subscriber >")
                    && input_msg_str.contains("InputMessage"))
                    || (output_msg_str.contains("as async_pub_sub :: Subscriber >")
                        && output_msg_str.contains("OutputMessage"))
            })
            .count();

        // If we have multiple generic fields without explicit constraints, this will cause conflicts
        if generic_associated_type_count > 1 {
            return Err(syn::Error::new_spanned(
                input,
                "Multiple generic subscribers detected without explicit message type constraints, which would create conflicting trait implementations.\n\nTip: When using multiple generic subscribers, you must specify explicit message types to avoid conflicts:\n\nInstead of:\n  struct MyStruct<A, B>\n  where\n      A: Subscriber,\n      B: Subscriber,\n\nUse one of these approaches:\n  1. Explicit message type constraints:\n     struct MyStruct<A, B>\n     where\n         A: Subscriber<InputMessage = i32, OutputMessage = String>,\n         B: Subscriber<InputMessage = bool, OutputMessage = f64>,\n\n  2. Use #[subscriber] attributes with explicit types:\n     struct MyStruct {\n         #[subscriber(i32, String)] field_a: A,\n         #[subscriber(bool, f64)] field_b: B,\n     }",
            ));
        }

        Ok(())
    }

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
    input_message_type: proc_macro2::TokenStream,
    output_message_type: proc_macro2::TokenStream,
}

impl Debug for SubscriberField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubscriberField")
            .field("name", &self.name.to_string())
            .field("input_message_type", &self.input_message_type.to_string())
            .field("output_message_type", &self.output_message_type.to_string())
            .finish()
    }
}

impl SubscriberField {
    fn new(
        name: proc_macro2::TokenStream,
        input_message_type: proc_macro2::TokenStream,
        output_message_type: proc_macro2::TokenStream,
    ) -> Self {
        Self {
            name,
            input_message_type,
            output_message_type,
        }
    }

    fn from_field_attributes_opt(field: &syn::Field) -> Option<Self> {
        let attr = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("subscriber"))?;

        let field_name = field.ident.clone().map(|ident| quote! { #ident })?;

        // Try to parse as a tuple of two types (input, output)
        if let Ok(tuple) = attr.parse_args::<syn::TypeTuple>() {
            if tuple.elems.len() == 2 {
                let input_type = &tuple.elems[0];
                let output_type = &tuple.elems[1];
                return Some(Self::new(
                    field_name,
                    quote! { #input_type },
                    quote! { #output_type },
                ));
            } else if !tuple.elems.is_empty() {
                // Invalid number of tuple elements
                return None;
            }
        }

        // Fall back to single type (same for input and output)
        if let Ok(ty) = attr.parse_args::<syn::Type>() {
            let message_type = quote! { #ty };
            return Some(Self::new(field_name, message_type.clone(), message_type));
        }

        None
    }

    fn from_field_type_opt(
        field: &syn::Field,
        generic_subscribers: &HashMap<
            syn::Ident,
            (proc_macro2::TokenStream, proc_macro2::TokenStream),
        >,
    ) -> Option<Self> {
        let syn::Type::Path(syn::TypePath { path, .. }) = &field.ty else {
            return None;
        };

        let ident = path.get_ident()?;

        let (input_message_type, output_message_type) =
            if let Some(message_types) = generic_subscribers.get(ident).cloned() {
                // For generic cases, use the tuple of input and output message types
                message_types
            } else {
                // For explicit path cases, get both input and output message types
                message_type_from_path_opt(path, "Subscriber")?
            };

        let field_name = field.ident.clone().map(|ident| quote! { #ident })?;

        Some(Self::new(
            field_name,
            input_message_type,
            output_message_type,
        ))
    }
}

impl SubscriberField {
    fn generate_impl(&self, context: &CodeGenerationContext) -> proc_macro2::TokenStream {
        let field_name = &self.name;
        let input_message_type = &self.input_message_type;
        let output_message_type = &self.output_message_type;
        let struct_name = &context.struct_name;
        let (impl_generics, ty_generics, where_clause) = &context.generics.split_for_impl();

        quote! {
            impl #impl_generics async_pub_sub::Subscriber for #struct_name #ty_generics #where_clause {
                type InputMessage = #input_message_type;
                type OutputMessage = #output_message_type;

                fn get_name(&self) -> &'static str {
                    async_pub_sub::Subscriber::get_name(&self.#field_name)
                }

                fn subscribe_to<P, Input>(&mut self, publisher: &mut P) -> async_pub_sub::Result<()>
                where
                    P: async_pub_sub::PublisherWrapper<Input, Self::InputMessage>,
                    Input: Send + 'static,
                {
                    async_pub_sub::Subscriber::subscribe_to(&mut self.#field_name, publisher)
                }

                fn receive(&mut self) -> async_pub_sub::futures::future::BoxFuture<'_, Self::OutputMessage> {
                    async_pub_sub::Subscriber::receive(&mut self.#field_name)
                }
            }
        }
    }

    fn generate_wrapper_impl(&self, context: &CodeGenerationContext) -> proc_macro2::TokenStream {
        let field_name = &self.name;
        let input_message_type = &self.input_message_type;
        let output_message_type = &self.output_message_type;
        let struct_name = &context.struct_name;
        let (impl_generics, ty_generics, where_clause) = &context.generics.split_for_impl();

        quote! {
            impl #impl_generics async_pub_sub::SubscriberWrapper<#input_message_type, #output_message_type>
            for #struct_name #ty_generics #where_clause {
                fn get_subscriber(&self) -> &impl async_pub_sub::Subscriber<InputMessage = #input_message_type, OutputMessage = #output_message_type> {
                    &self.#field_name
                }

                fn get_subscriber_mut(&mut self) -> &mut impl async_pub_sub::Subscriber<InputMessage = #input_message_type, OutputMessage = #output_message_type> {
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
                    input,
                    "DeriveSubscriber macro only supports structs with named fields.\n\nTip: Change your struct to use named fields like `struct MyStruct { field: Type }` instead of tuple structs like `struct MyStruct(Type)`.",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "DeriveSubscriber macro only supports structs.\n\nTip: The macro can only be applied to struct definitions, not enums, unions, or other types. Use `struct MyStruct { ... }` instead.",
            ));
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
