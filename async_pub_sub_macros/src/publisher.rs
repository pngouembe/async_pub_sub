use proc_macro::TokenStream;
use quote::quote;
use std::{collections::HashMap, fmt::Debug};
use syn::DeriveInput;

use crate::helpers::{find_pub_sub_types_in_generics, message_type_from_path_opt};

pub(crate) fn derive_publisher_impl(input: DeriveInput) -> TokenStream {
    InputStruct::try_from(input)
        .map(|input| input.generate_code())
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
                "DerivePublisher macro requires that your struct must have at least one field that implements the Publisher trait or is marked with #[publisher].\n\nTip: You can either:\n  1. Add a field with type that implements Publisher: `field: SomePublisher`\n  2. Add #[publisher] attribute to a field: `#[publisher(MessageType)] field: SomeType`\n  3. Add #[publisher] attribute with input/output types: `#[publisher(InputType, OutputType)] field: SomeType`\n  4. Use generic constraints: `struct MyStruct<P: Publisher<InputMessage = i32, OutputMessage = String>>`",
            ));
        }

        let is_multi_publisher = publisher_fields.len() > 1;
        
        // Check for conflicting generic publishers without explicit message type constraints
        if is_multi_publisher {
            Self::check_for_conflicting_generic_types(&publisher_fields, &input)?;
        }
        
        Ok(Self {
            context,
            publisher_fields,
            is_multi_publisher,
        })
    }
}

impl InputStruct {
    fn check_for_conflicting_generic_types(
        publisher_fields: &[PublisherField],
        input: &DeriveInput,
    ) -> Result<(), syn::Error> {
        // Count how many fields use generic associated types without explicit message type constraints
        let generic_associated_type_count = publisher_fields
            .iter()
            .filter(|field| {
                // Check if this field uses generic associated types like <T as Publisher>::InputMessage
                let input_msg_str = field.input_message_type.to_string();
                let output_msg_str = field.output_message_type.to_string();
                
                // Pattern: <SomeGeneric as async_pub_sub::Publisher>::InputMessage
                (input_msg_str.contains("as async_pub_sub :: Publisher >") && 
                 input_msg_str.contains("InputMessage")) ||
                (output_msg_str.contains("as async_pub_sub :: Publisher >") && 
                 output_msg_str.contains("OutputMessage"))
            })
            .count();

        // If we have multiple generic fields without explicit constraints, this will cause conflicts
        if generic_associated_type_count > 1 {
            return Err(syn::Error::new_spanned(
                input,
                "Multiple generic publishers detected without explicit message type constraints, which would create conflicting trait implementations.\n\nTip: When using multiple generic publishers, you must specify explicit message types to avoid conflicts:\n\nInstead of:\n  struct MyStruct<A, B>\n  where\n      A: Publisher,\n      B: Publisher,\n\nUse one of these approaches:\n  1. Explicit message type constraints:\n     struct MyStruct<A, B>\n     where\n         A: Publisher<InputMessage = i32, OutputMessage = String>,\n         B: Publisher<InputMessage = bool, OutputMessage = f64>,\n\n  2. Use #[publisher] attributes with explicit types:\n     struct MyStruct {\n         #[publisher(i32, String)] field_a: A,\n         #[publisher(bool, f64)] field_b: B,\n     }",
            ));
        }
        
        Ok(())
    }

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
    input_message_type: proc_macro2::TokenStream,
    output_message_type: proc_macro2::TokenStream,
}

impl Debug for PublisherField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PublisherField")
            .field("name", &self.name.to_string())
            .field("input_message_type", &self.input_message_type.to_string())
            .field("output_message_type", &self.output_message_type.to_string())
            .finish()
    }
}

impl PublisherField {
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
            .find(|attr| attr.path().is_ident("publisher"))?;

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
        generic_publishers: &HashMap<syn::Ident, (proc_macro2::TokenStream, proc_macro2::TokenStream)>,
    ) -> Option<Self> {
        let syn::Type::Path(syn::TypePath { path, .. }) = &field.ty else {
            return None;
        };

        let ident = path.get_ident()?;

        let (input_message_type, output_message_type) = if let Some(message_types) = generic_publishers.get(ident).cloned() {
            // For generic cases, use the tuple of input and output message types
            message_types
        } else {
            // For explicit path cases, get both input and output message types
            message_type_from_path_opt(path, "Publisher")?
        };

        let field_name = field.ident.clone().map(|ident| quote! { #ident })?;

        Some(Self::new(field_name, input_message_type, output_message_type))
    }

    fn generate_impl(&self, context: &CodeGenerationContext) -> proc_macro2::TokenStream {
        let field_name = &self.name;
        let input_message_type = &self.input_message_type;
        let output_message_type = &self.output_message_type;
        let struct_name = &context.struct_name;
        let (impl_generics, ty_generics, where_clause) = &context.generics.split_for_impl();

        quote! {
            impl #impl_generics async_pub_sub::Publisher for #struct_name #ty_generics #where_clause {
                type InputMessage = #input_message_type;
                type OutputMessage = #output_message_type;

                fn get_name(&self) -> &'static str {
                    async_pub_sub::Publisher::get_name(&self.#field_name)
                }

                fn publish(&self, message: Self::InputMessage) -> async_pub_sub::futures::future::BoxFuture<async_pub_sub::Result<()>> {
                    async_pub_sub::Publisher::publish(&self.#field_name, message)
                }

                fn get_message_stream(
                    &mut self,
                    subscriber_name: &'static str,
                ) -> async_pub_sub::Result<std::pin::Pin<Box<dyn async_pub_sub::futures::Stream<Item = Self::OutputMessage> + Send + Sync + 'static>>> {
                    async_pub_sub::Publisher::get_message_stream(&mut self.#field_name, subscriber_name)
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
            impl #impl_generics async_pub_sub::PublisherWrapper<#input_message_type, #output_message_type>
            for #struct_name #ty_generics #where_clause {
                fn get_publisher(&self) -> &impl async_pub_sub::Publisher<InputMessage = #input_message_type, OutputMessage = #output_message_type> {
                    &self.#field_name
                }

                fn get_publisher_mut(&mut self) -> &mut impl async_pub_sub::Publisher<InputMessage = #input_message_type, OutputMessage = #output_message_type> {
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
                    input,
                    "DerivePublisher macro only supports structs with named fields.\n\nTip: Change your struct to use named fields like `struct MyStruct { field: Type }` instead of tuple structs like `struct MyStruct(Type)`.",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "DerivePublisher macro only supports structs.\n\nTip: The macro can only be applied to struct definitions, not enums, unions, or other types. Use `struct MyStruct { ... }` instead.",
            ));
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
