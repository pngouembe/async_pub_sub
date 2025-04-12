use std::fmt::Debug;
use futures::Future;

use crate::{utils::Layer, PublisherWrapper, Result, Subscriber};

/// A subscriber middleware layer that adds debug logging capabilities.
/// This layer will log all messages using debug format when they are received.
pub struct DebuggingSubscriberLayer;

impl<S> Layer<S> for DebuggingSubscriberLayer
where
    S: Subscriber + Send,
    S::Message: Debug,
{
    type LayerType = DebugSubscriber<S>;

    fn layer(&self, subscriber: S) -> Self::LayerType {
        DebugSubscriber {
            publisher_name: None,
            subscriber,
        }
    }
}

/// A subscriber wrapper that adds debug logging capabilities to an existing subscriber.
/// Logs all messages using debug format when they are received.
#[derive(Debug)]
pub struct DebugSubscriber<S>
where
    S: Subscriber + Send,
{
    /// The name of the publisher sending messages, set when subscribe_to is called
    publisher_name: Option<&'static str>,
    /// The underlying subscriber being wrapped
    subscriber: S,
}

impl<S> Subscriber for DebugSubscriber<S>
where
    S: Subscriber + Send,
    S::Message: Debug,
{
    type Message = S::Message;

    fn get_name(&self) -> &'static str {
        self.subscriber.get_name()
    }

    fn subscribe_to(&mut self, publisher: &mut impl PublisherWrapper<Self::Message>) -> Result<()> {
        self.publisher_name = Some(publisher.get_name());
        log::info!(
            "({}) <-> ({})",
            self.subscriber.get_name(),
            publisher.get_name(),
        );
        self.subscriber.subscribe_to(publisher)
    }

    fn receive(&mut self) -> impl Future<Output = Self::Message> + Send {
        let publisher_name = self.publisher_name.expect("publisher name should be known");
        let subscriber_name = self.subscriber.get_name();
        
        async move {
            let message = self.subscriber.receive().await;
            log::info!(
                "[{}] <- [{}]: {:?}",
                subscriber_name,
                publisher_name,
                message
            );
            message
        }
    }
}