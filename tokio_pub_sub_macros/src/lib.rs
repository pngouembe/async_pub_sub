mod helpers;
mod publisher;
mod rpc;
mod subscriber;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_derive(DeriveSubscriber)]
pub fn derive_subscriber(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    subscriber::derive_subscriber_impl(input)
}

#[proc_macro_derive(DerivePublisher, attributes(publisher))]
pub fn derive_publisher(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    publisher::derive_publisher_impl(input)
}

#[proc_macro_attribute]
pub fn rpc_interface(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::Item);
    rpc::generate_rpc_interface(input)
}
