//! Asynchronous publish-subscribe macro library for Rust.
#![doc = include_str!("../README.md")]

mod helpers;
mod publisher;
mod route;
mod rpc;
mod subscriber;

use proc_macro::TokenStream;
use syn::parse_macro_input;

/// Automatically implements the `Subscriber` trait for a struct.
///
/// This macro can be used in three ways:
/// 1. For a single subscriber field - implements the `Subscriber` trait directly
/// 2. For multiple subscriber fields - implements the `SubscriberWrapper` trait for each field
/// 3. For fields marked with the `#[subscriber(type)]` attribute - specifies the message type
///
/// # Examples
/// ```rust
/// use async_pub_sub::Subscriber;
/// use async_pub_sub::SubscriberImpl;
/// use async_pub_sub_macros::DeriveSubscriber;
///
/// // Single subscriber
/// #[derive(DeriveSubscriber)]
/// struct MySubscriber<S: Subscriber> {
///     subscriber: S,
/// }
///
/// // Multiple subscribers
/// #[derive(DeriveSubscriber)]
/// struct MultiSubscriber<A, B> where
///     A: Subscriber<Message = i32>,
///     B: Subscriber<Message = String>,
/// {
///     subscriber_a: A,
///     subscriber_b: B,
/// }
///
/// // Using attribute to specify message type
/// #[derive(DeriveSubscriber)]
/// struct TypedSubscriber {
///     #[subscriber(i32)]
///     subscriber: SubscriberImpl<i32>,
/// }
/// ```
#[proc_macro_derive(DeriveSubscriber, attributes(subscriber))]
pub fn derive_subscriber(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    subscriber::derive_subscriber_impl(input)
}

/// Automatically implements the `Publisher` trait for a struct.
///
/// This macro can be used in three ways:
/// 1. For a single publisher field - implements the `Publisher` trait directly
/// 2. For multiple publisher fields - implements the `PublisherWrapper` trait for each field
/// 3. For fields marked with the `#[publisher(type)]` attribute - specifies the message type
///
/// # Examples
/// ```rust
/// use async_pub_sub::{Publisher, PublisherImpl};
/// use async_pub_sub_macros::DerivePublisher;
///
/// // Using a generic publisher
/// #[derive(DerivePublisher)]
/// struct MyPublisher<P: Publisher> {
///     publisher: P,
/// }
///
/// // Multiple publishers
/// #[derive(DerivePublisher)]
/// struct MultiPublisher<A, B> where
///     A: Publisher<Message = i32>,
///     B: Publisher<Message = String>,
/// {
///     publisher_a: A,
///     publisher_b: B,
/// }
///
/// // Using attribute to specify message type
/// #[derive(DerivePublisher)]
/// struct TypedPublisher {
///     #[publisher(i32)]
///     publisher: PublisherImpl<i32>,
/// }
/// ```
#[proc_macro_derive(DerivePublisher, attributes(publisher))]
pub fn derive_publisher(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    publisher::derive_publisher_impl(input)
}

/// Generates the necessary code for defining RPC interfaces.
///
/// This macro generates:
/// - Message enums for RPC communication (with optional derive attributes)
/// - Client traits
/// - Server traits
///
/// # Examples
/// ```rust
/// use async_pub_sub_macros::rpc_interface;
///
/// // Basic usage
/// #[rpc_interface]
/// trait MyRpcInterface {
///     async fn my_method(&self, arg: i32) -> String;
/// }
///
/// // With derive attributes for the generated message enum
/// #[rpc_interface(Debug)]
/// trait MyLoggableRpcInterface {
///     async fn get_data(&self) -> String;
///     async fn set_data(&mut self, data: String);
/// }
/// ```
///
/// The derive attributes provided to the macro will be applied to the generated
/// message enum. Common derive attributes.
/// As an example, you might want to add the `Debug` derive attribute to the
/// generated message enum to enable debugging the communication between you client and you server.
#[proc_macro_attribute]
pub fn rpc_interface(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::Item);
    rpc::generate_rpc_interface(attr, input)
}

/// Creates a connection between a single publisher and subscriber.
///
/// # Examples
/// ```rust
/// use async_pub_sub_macros::route;
/// use async_pub_sub::{PublisherImpl, SubscriberImpl};
///
/// // Define your publisher and subscriber
/// let mut publisher = PublisherImpl::new("publisher", 1);
/// let mut subscriber = SubscriberImpl::new("subscriber");
///
/// route!(publisher -> subscriber: i32).unwrap();
/// ```
#[proc_macro]
pub fn route(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as route::RouteInput);
    route::generate_route(input)
}

/// Creates connections between multiple publishers and subscribers.
///
/// # Examples
/// ```rust
/// use async_pub_sub_macros::routes;
/// use async_pub_sub::{PublisherImpl, SubscriberImpl};
///
/// // Define your publishers and subscribers
/// let mut publisher_a = PublisherImpl::<i32>::new("publisher_a", 1);
/// let mut publisher_b = PublisherImpl::new("publisher_b", 1);
/// let mut subscriber_a = SubscriberImpl::new("subscriber_a");
/// let mut subscriber_b = SubscriberImpl::new("subscriber_b");
///
/// routes! {
///     publisher_a -> subscriber_a,
///     publisher_b -> subscriber_b: String,
/// }.unwrap();
/// ```
#[proc_macro]
pub fn routes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as route::RoutesInput);
    route::generate_routes(input)
}
