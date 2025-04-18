use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Token, Type};

pub struct RouteInput {
    subscriber: syn::Expr,
    publisher: proc_macro2::TokenStream,
    message_type: Option<Type>,
}

impl Parse for RouteInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let publisher = parse_publisher_token_stream(input)?;
        input.parse::<Token![->]>()?;

        let subscriber = input.parse()?;

        let message_type = if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(RouteInput {
            subscriber,
            publisher,
            message_type,
        })
    }
}

fn parse_publisher_token_stream(input: ParseStream) -> syn::Result<proc_macro2::TokenStream> {
    let publisher_start = input.cursor();

    input.step(|cursor| {
        let mut rest = *cursor;
        while let Some((tt, next)) = rest.token_tree() {
            if let TokenTree::Punct(punct) = &tt {
                if punct.as_char() == '-' {
                    if let Some((TokenTree::Punct(punct), _)) = next.token_tree() {
                        if punct.as_char() == '>' {
                            return Ok(((), rest));
                        }
                    }
                }
            }
            rest = next;
        }
        Err(cursor.error("Expected '->'"))
    })?;

    let publisher_end = input.cursor();

    let mut cursor = publisher_start;
    let mut publisher = proc_macro2::TokenStream::new();
    while cursor < publisher_end {
        let (token, next) = cursor.token_tree().unwrap();
        publisher.extend(std::iter::once(token));
        cursor = next;
    }

    Ok(publisher)
}

pub(crate) fn generate_route(input: RouteInput) -> TokenStream {
    let subscriber = input.subscriber;
    let publisher = input.publisher;

    let output = if let Some(message_type) = input.message_type {
        quote! {
            async_pub_sub::SubscriberWrapper::<#message_type>::subscribe_to(
                &mut #subscriber,
                async_pub_sub::PublisherWrapper::<_>::get_publisher_mut(&mut #publisher),
            )
        }
    } else {
        quote! {
            async_pub_sub::SubscriberWrapper::<_>::subscribe_to(
                &mut #subscriber,
                async_pub_sub::PublisherWrapper::<_>::get_publisher_mut(&mut #publisher),
            )
        }
    };

    output.into()
}

pub struct RoutesInput {
    routes: Vec<RouteInput>,
}

impl Parse for RoutesInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let routes = input
            .parse_terminated(RouteInput::parse, Token![,])?
            .into_iter()
            .collect();

        Ok(RoutesInput { routes })
    }
}

pub(crate) fn generate_routes(input: RoutesInput) -> TokenStream {
    let routes = input.routes.into_iter().map(|route| {
        let route: proc_macro2::TokenStream = generate_route(route).into();
        quote! {
            .and_then(|_| {#route})
        }
    });

    quote! {
        Ok(())#(#routes)*
    }
    .into()
}
