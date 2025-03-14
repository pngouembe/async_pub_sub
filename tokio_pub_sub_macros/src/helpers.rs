use syn::{DeriveInput, GenericParam, Type, TypeParamBound, TypePath};

pub(crate) fn find_subscriber_field<'a>(
    fields: &'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    input: &'a DeriveInput,
) -> Option<&'a syn::Field> {
    fields
        .iter()
        .find(|field| has_subscriber_bound(field, input))
}

fn has_subscriber_bound(field: &syn::Field, input: &DeriveInput) -> bool {
    if let Type::Path(TypePath { path, .. }) = &field.ty {
        let type_name = path.segments.first().map(|s| &s.ident);
        input.generics.params.iter().any(|p| {
            if let GenericParam::Type(type_param) = p {
                if Some(&type_param.ident) == type_name {
                    check_trait_bounds(type_param, input)
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

fn check_trait_bounds(type_param: &syn::TypeParam, input: &DeriveInput) -> bool {
    let has_subscriber_bound = type_param.bounds.iter().any(is_subscriber_bound);

    let has_where_bound = input
        .generics
        .where_clause
        .as_ref()
        .map(|where_clause| check_where_clause(where_clause, &type_param.ident))
        .unwrap_or(false);

    has_subscriber_bound || has_where_bound
}

fn check_where_clause(where_clause: &syn::WhereClause, type_ident: &syn::Ident) -> bool {
    where_clause.predicates.iter().any(|pred| {
        if let syn::WherePredicate::Type(pred_type) = pred {
            if let Type::Path(TypePath { path, .. }) = &pred_type.bounded_ty {
                path.segments
                    .first()
                    .map(|s| s.ident == *type_ident)
                    .unwrap_or(false)
                    && pred_type.bounds.iter().any(is_subscriber_bound)
            } else {
                false
            }
        } else {
            false
        }
    })
}

fn is_subscriber_bound(bound: &TypeParamBound) -> bool {
    matches!(bound, TypeParamBound::Trait(t) if t.path.segments.last()
        .map(|s| s.ident == "Subscriber")
        .unwrap_or(false))
}

pub(crate) fn find_all_publisher_fields<'a>(
    fields: &'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    input: &'a DeriveInput,
) -> Vec<&'a syn::Field> {
    fields
        .iter()
        .filter(|field| has_publisher_bound(field, input))
        .collect()
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
