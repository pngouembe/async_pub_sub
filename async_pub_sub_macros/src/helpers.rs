use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    AngleBracketedGenericArguments, DeriveInput, GenericParam, Type, TypeParamBound, TypePath,
};

pub(crate) fn find_all_publisher_fields<'a>(
    fields: &'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    input: &'a DeriveInput,
) -> Vec<(&'a syn::Field, TokenStream)> {
    fields
        .iter()
        .filter_map(|field| {
            if has_publisher_bound(field, input) {
                Some((field, get_generic_publisher_message_type(field, input)))
            } else if has_publisher_attribute(field) {
                Some((field, get_concrete_publisher_message_type(field)))
            } else {
                None
            }
        })
        .collect()
}

fn get_generic_publisher_message_type(field: &syn::Field, input: &DeriveInput) -> TokenStream {
    let type_param = if let Type::Path(TypePath { path, .. }) = &field.ty {
        path
    } else {
        panic!("Invalid field type")
    };

    if let Some(message_type) = find_publisher_message_type(type_param) {
        quote! { #message_type }
    } else if let Some(message_type) = find_publisher_message_type_in_bounds(type_param, input) {
        quote! { #message_type }
    } else {
        quote! { <#type_param as async_pub_sub::Publisher>::Message }
    }
}

fn find_publisher_message_type(type_param: &syn::Path) -> Option<TokenStream> {
    let message_type = type_param
        .segments
        .iter()
        .find(|segment| {
            segment.ident == "Publisher"
                && matches!(segment.arguments, syn::PathArguments::AngleBracketed(_))
        })
        .map(|segment| {
            if let syn::PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                args, ..
            }) = &segment.arguments
            {
                if let Some(syn::GenericArgument::AssocType(assoc_type)) = args.first() {
                    if assoc_type.ident == "Message" {
                        let message_type = assoc_type.ty.clone();
                        return Some(quote! { #message_type });
                    }
                }
            }
            None
        });

    message_type.flatten()
}

fn find_publisher_message_type_in_bounds(
    type_param: &syn::Path,
    input: &DeriveInput,
) -> Option<TokenStream> {
    let publisher_type = type_param.segments.first().map(|s| &s.ident);

    // First check generic parameters
    for param in &input.generics.params {
        if let GenericParam::Type(type_param) = param {
            if Some(&type_param.ident) == publisher_type {
                for bound in &type_param.bounds {
                    if let TypeParamBound::Trait(t) = bound {
                        if t.path
                            .segments
                            .last()
                            .map(|s| s.ident == "Publisher")
                            .unwrap_or(false)
                        {
                            if let syn::PathArguments::AngleBracketed(args) =
                                &t.path.segments.last().unwrap().arguments
                            {
                                if let Some(syn::GenericArgument::AssocType(assoc_type)) =
                                    args.args.first()
                                {
                                    if assoc_type.ident == "Message" {
                                        let message_type = assoc_type.ty.clone();
                                        return Some(quote! { #message_type });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Then check where clause
    if let Some(where_clause) = &input.generics.where_clause {
        for predicate in &where_clause.predicates {
            if let syn::WherePredicate::Type(pred_type) = predicate {
                if let Type::Path(TypePath { path, .. }) = &pred_type.bounded_ty {
                    if path.segments.first().map(|s| &s.ident) == publisher_type {
                        for bound in &pred_type.bounds {
                            if let TypeParamBound::Trait(t) = bound {
                                if t.path
                                    .segments
                                    .last()
                                    .map(|s| s.ident == "Publisher")
                                    .unwrap_or(false)
                                {
                                    if let syn::PathArguments::AngleBracketed(args) =
                                        &t.path.segments.last().unwrap().arguments
                                    {
                                        if let Some(syn::GenericArgument::AssocType(assoc_type)) =
                                            args.args.first()
                                        {
                                            if assoc_type.ident == "Message" {
                                                let message_type = assoc_type.ty.clone();
                                                return Some(quote! { #message_type });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

fn get_concrete_publisher_message_type(field: &syn::Field) -> TokenStream {
    if let Some(attr) = field
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("publisher"))
    {
        attr.parse_args()
            .expect("Expected a type parameter for #[publisher]")
    } else {
        panic!("Should not call this function on a field that is not decorated with the publisher attribute")
    }
}

fn has_publisher_bound(field: &syn::Field, input: &DeriveInput) -> bool {
    if let Type::Path(TypePath { path, .. }) = &field.ty {
        let type_name = path.segments.first().map(|s| &s.ident);
        input.generics.params.iter().any(|p| {
            if let GenericParam::Type(type_param) = p {
                if Some(&type_param.ident) == type_name {
                    check_publisher_trait_bounds(type_param, input)
                } else {
                    false
                }
            } else {
                false
            }
        })
    } else {
        false
    }
}

fn has_publisher_attribute(field: &syn::Field) -> bool {
    field
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("publisher"))
}

fn check_publisher_trait_bounds(type_param: &syn::TypeParam, input: &DeriveInput) -> bool {
    let has_publisher_bound = type_param.bounds.iter().any(is_publisher_bound);

    let has_where_bound = input
        .generics
        .where_clause
        .as_ref()
        .map(|where_clause| check_publisher_where_clause(where_clause, &type_param.ident))
        .unwrap_or(false);

    has_publisher_bound || has_where_bound
}

fn check_publisher_where_clause(where_clause: &syn::WhereClause, type_ident: &syn::Ident) -> bool {
    where_clause.predicates.iter().any(|pred| {
        if let syn::WherePredicate::Type(pred_type) = pred {
            if let Type::Path(TypePath { path, .. }) = &pred_type.bounded_ty {
                path.segments
                    .first()
                    .map(|s| s.ident == *type_ident)
                    .unwrap_or(false)
                    && pred_type.bounds.iter().any(is_publisher_bound)
            } else {
                false
            }
        } else {
            false
        }
    })
}

fn is_publisher_bound(bound: &TypeParamBound) -> bool {
    matches!(bound, TypeParamBound::Trait(t) if t.path.segments.last()
        .map(|s| s.ident == "Publisher")
        .unwrap_or(false))
}
