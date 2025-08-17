//! Asynchronous publish-subscribe library for Rust.
#![doc = include_str!("../README.md")]

mod publisher;
mod request;
mod subscriber;
mod utils;

mod error;

pub use error::{Error, Result};

pub use publisher::{
    DebuggingPublisherLayer, LoggingPublisherLayer, MappedPublisher, Publisher, PublisherBuilder,
    PublisherImpl, PublisherMapLayer, PublisherRequestMapLayer, PublisherRequestMapPublisher,
    PublisherWrapper,
};
pub use request::{
    LoggingRequestLayer, MappedRequest, Request, RequestBuilder, RequestImpl, Requester,
};
pub use subscriber::{
    DebuggingSubscriberLayer, LoggingSubscriberLayer, Subscriber, SubscriberBuilder,
    SubscriberImpl, SubscriberMapLayer, SubscriberMapSubscriber, SubscriberRequestMapLayer,
    SubscriberRequestMapSubscriber, SubscriberRequestMapWrapper, SubscriberWrapper,
};
pub use utils::{DebuggingForwarderLayer, Forwarder, ForwarderBuilder, ForwarderImpl, Layer};

// Re-export futures for use in macros and client code
pub use futures;

#[cfg(feature = "macros")]
pub use async_pub_sub_macros as macros;
