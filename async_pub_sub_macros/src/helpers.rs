use std::collections::HashMap;

use quote::quote;

pub fn find_pub_sub_types_in_generics(
    trait_name: &str,
    generics: &syn::Generics,
) -> HashMap<syn::Ident, (proc_macro2::TokenStream, proc_macro2::TokenStream)> {
    let mut pub_sub_types = find_pub_sub_types_in_generic_type_params(trait_name, generics);
    pub_sub_types.extend(find_pub_sub_types_in_where_clauses(trait_name, generics));
    pub_sub_types
}

fn find_pub_sub_types_in_generic_type_params(
    trait_name: &str,
    generics: &syn::Generics,
) -> HashMap<syn::Ident, (proc_macro2::TokenStream, proc_macro2::TokenStream)> {
    generics
        .type_params()
        .filter_map(|type_param| pub_sub_type_entry_from_type_param_opt(type_param, trait_name))
        .collect()
}

fn find_pub_sub_types_in_where_clauses(
    trait_name: &str,
    generics: &syn::Generics,
) -> HashMap<syn::Ident, (proc_macro2::TokenStream, proc_macro2::TokenStream)> {
    generics
        .where_clause
        .as_ref()
        .map(|where_clause| {
            where_clause
                .predicates
                .iter()
                .filter_map(|predicate| {
                    pub_sub_type_entry_from_where_predicate_opt(predicate, trait_name)
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn pub_sub_type_entry_from_type_param_opt(
    type_param: &syn::TypeParam,
    trait_name: &str,
) -> Option<(syn::Ident, (proc_macro2::TokenStream, proc_macro2::TokenStream))> {
    let syn::TypeParam { ident, bounds, .. } = type_param;

    let trait_ident = proc_macro2::Ident::new(trait_name, proc_macro2::Span::call_site());

    bounds.iter().find_map(|bound| {
        let syn::TypeParamBound::Trait(trait_bound) = bound else {
            return None;
        };

        let path = &trait_bound.path;

        if path.is_ident(trait_name) {
            let input_message = quote! { <#ident as async_pub_sub::#trait_ident>::InputMessage };
            let output_message = quote! { <#ident as async_pub_sub::#trait_ident>::OutputMessage };
            return Some((
                ident.clone(),
                (input_message, output_message),
            ));
        }

        let message_types = bounds.iter().find_map(|bound| {
            let syn::TypeParamBound::Trait(trait_bound) = bound else {
                return None;
            };

            let path = &trait_bound.path;

            if path.is_ident(trait_name) {
                let input_message = quote! { <#ident as async_pub_sub::#trait_ident>::InputMessage };
                let output_message = quote! { <#ident as async_pub_sub::#trait_ident>::OutputMessage };
                return Some((input_message, output_message));
            }

            path.segments
                .iter()
                .find_map(message_type_from_path_segment_opt)
                .or_else(|| {
                    let input_message = quote! { <#ident as async_pub_sub::#trait_ident>::InputMessage };
                    let output_message = quote! { <#ident as async_pub_sub::#trait_ident>::OutputMessage };
                    Some((input_message, output_message))
                })
        })?;

        Some((type_param.ident.clone(), message_types))
    })
}

pub fn pub_sub_type_entry_from_where_predicate_opt(
    predicate: &syn::WherePredicate,
    trait_name: &str,
) -> Option<(syn::Ident, (proc_macro2::TokenStream, proc_macro2::TokenStream))> {
    let trait_ident = proc_macro2::Ident::new(trait_name, proc_macro2::Span::call_site());

    let syn::WherePredicate::Type(syn::PredicateType {
        bounded_ty, bounds, ..
    }) = predicate
    else {
        return None;
    };

    let syn::Type::Path(syn::TypePath { path, .. }) = bounded_ty else {
        return None;
    };

    let ident = path.get_ident()?;

    let message_types = bounds.iter().find_map(|bound| {
        let syn::TypeParamBound::Trait(trait_bound) = bound else {
            return None;
        };

        let path = &trait_bound.path;

        if path.is_ident(trait_name) {
            let input_message = quote! { <#ident as async_pub_sub::#trait_ident>::InputMessage };
            let output_message = quote! { <#ident as async_pub_sub::#trait_ident>::OutputMessage };
            return Some((input_message, output_message));
        }

        path.segments
            .iter()
            .find_map(message_type_from_path_segment_opt)
            .or_else(|| {
                let input_message = quote! { <#ident as async_pub_sub::#trait_ident>::InputMessage };
                let output_message = quote! { <#ident as async_pub_sub::#trait_ident>::OutputMessage };
                Some((input_message, output_message))
            })
    })?;

    Some((ident.clone(), message_types))
}

pub fn message_type_from_path_opt(
    path: &syn::Path,
    trait_name: &str,
) -> Option<(proc_macro2::TokenStream, proc_macro2::TokenStream)> {
    let trait_ident = proc_macro2::Ident::new(trait_name, proc_macro2::Span::call_site());

    let ident = path.get_ident()?;

    if ident != trait_name {
        return None;
    }

    if path.is_ident(trait_name) {
        let input_message = quote! { <#ident as async_pub_sub::#trait_ident>::InputMessage };
        let output_message = quote! { <#ident as async_pub_sub::#trait_ident>::OutputMessage };
        return Some((input_message, output_message));
    }

    path.segments
        .iter()
        .find_map(message_type_from_path_segment_opt)
        .or_else(|| {
            let input_message = quote! { <#ident as async_pub_sub::#trait_ident>::InputMessage };
            let output_message = quote! { <#ident as async_pub_sub::#trait_ident>::OutputMessage };
            Some((input_message, output_message))
        })
}

fn message_type_from_path_segment_opt(
    segment: &syn::PathSegment,
) -> Option<(proc_macro2::TokenStream, proc_macro2::TokenStream)> {
    let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments { args, .. }) =
        &segment.arguments
    else {
        return None;
    };

    let mut input_message = None;
    let mut output_message = None;

    // Look for InputMessage and OutputMessage associated types
    for arg in args.iter() {
        let syn::GenericArgument::AssocType(assoc_ty) = arg else {
            continue;
        };

        match assoc_ty.ident.to_string().as_str() {
            "InputMessage" => {
                let ty = &assoc_ty.ty;
                input_message = Some(quote! { #ty });
            }
            "OutputMessage" => {
                let ty = &assoc_ty.ty;
                output_message = Some(quote! { #ty });
            }
            _ => {}
        }
    }

    match (input_message, output_message) {
        (Some(input), Some(output)) => Some((input, output)),
        _ => None,
    }
}
