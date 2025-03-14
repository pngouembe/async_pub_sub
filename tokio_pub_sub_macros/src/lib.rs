mod helpers;
mod subscriber;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_derive(DeriveSubscriber)]
pub fn derive_subscriber(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    subscriber::derive_subscriber_impl(input)
}
