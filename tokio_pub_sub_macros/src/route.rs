use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Token, Type};

pub struct RouteInput {
    subscriber: syn::Ident,
    publisher: syn::Ident,
    message_type: Option<Type>,
}

impl Parse for RouteInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let publisher = input.parse()?;
        input.parse::<Token![ -> ]>()?;
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

pub(crate) fn generate_route(input: RouteInput) -> TokenStream {
    let subscriber = input.subscriber;
    let publisher = input.publisher;

    let output = if let Some(message_type) = input.message_type {
        quote! {
            tokio_pub_sub::MultiSubscriber::<#message_type>::subscribe_to(&mut #subscriber, &mut #publisher)
        }
    } else {
        quote! {
            #subscriber.subscribe_to(&mut #publisher)
        }
    };

    output.into()
}
