use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, GenericParam,  Type, TypePath};

#[proc_macro_derive(DeriveSubscriber)]
pub fn derive_subscriber(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
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
    let subscriber_field = fields
        .iter()
        .find(|field| {
            if let Type::Path(TypePath { path, .. }) = &field.ty {
                let type_name = path.segments.first().map(|s| &s.ident);
                input.generics.params.iter().any(|p| {
                    if let GenericParam::Type(type_param) = p {
                        // Check if this is our field's type parameter
                        if Some(&type_param.ident) == type_name {
                            // Check bounds in both type parameter and where clause
                            let has_subscriber_bound = type_param.bounds.iter().any(|bound| {
                                matches!(bound, syn::TypeParamBound::Trait(t) if t.path.segments.last()
                                    .map(|s| s.ident == "Subscriber")
                                    .unwrap_or(false))
                            });
                            
                            let has_where_bound = input.generics.where_clause.as_ref()
                                .map(|where_clause| {
                                    where_clause.predicates.iter().any(|pred| {
                                        if let syn::WherePredicate::Type(pred_type) = pred {
                                            if let Type::Path(TypePath { path, .. }) = &pred_type.bounded_ty {
                                                path.segments.first()
                                                    .map(|s| s.ident == type_param.ident)
                                                    .unwrap_or(false) 
                                                    && pred_type.bounds.iter().any(|bound| {
                                                        matches!(bound, syn::TypeParamBound::Trait(t) if t.path.segments.last()
                                                            .map(|s| s.ident == "Subscriber")
                                                            .unwrap_or(false))
                                                    })
                                            } else {
                                                false
                                            }
                                        } else {
                                            false
                                        }
                                    })
                                })
                                .unwrap_or(false);

                            return has_subscriber_bound || has_where_bound;
                        }
                    }
                    false
                })
            } else {
                false
            }
        })
        .expect("Struct must have a field that implements the Subscriber trait");

    let field_name = &subscriber_field.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Find the generic type parameter name from the field type
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
