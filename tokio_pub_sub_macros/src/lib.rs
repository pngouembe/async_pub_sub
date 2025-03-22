mod helpers;
mod publisher;
mod route;
mod rpc;
mod subscriber;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_derive(DeriveSubscriber, attributes(subscriber))]
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

#[proc_macro]
pub fn route(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as route::RouteInput);
    route::generate_route(input)
}
