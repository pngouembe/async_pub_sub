//! Asynchronous publish-subscribe library for Rust.
#![doc = include_str!("../README.md")]

mod publisher;
mod subscriber;
mod utils;

mod error;

pub use error::{Error, Result};

pub use publisher::{
    DebugingPublisherLayer, LoggingPublisherLayer, PublisherWrapper, Publisher, PublisherBuilder,
    PublisherImpl, PublisherLayer, Request,
};
pub use subscriber::{MultiSubscriber, Subscriber, SubscriberImpl};
pub use utils::LoggingForwarder;

#[cfg(feature = "macros")]
pub use async_pub_sub_macros as macros;

pub use futures;
