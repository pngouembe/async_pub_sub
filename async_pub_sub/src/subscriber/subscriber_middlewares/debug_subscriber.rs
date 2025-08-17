use futures::{FutureExt, future::BoxFuture};
use std::fmt::Debug;

use crate::{PublisherWrapper, Result, Subscriber, utils::Layer};

/// A subscriber middleware layer that adds debug logging capabilities.
/// This layer will log all messages using debug format when they are received.
pub struct DebuggingSubscriberLayer;

impl<S> Layer<S> for DebuggingSubscriberLayer
where
    S: Subscriber + Send,
    S::InputMessage: Debug,
{
    type LayerType = DebugSubscriber<S>;

    fn layer(self, subscriber: S) -> Self::LayerType {
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
    S::OutputMessage: Debug,
{
    type InputMessage = S::InputMessage;
    type OutputMessage = S::OutputMessage;

    fn get_name(&self) -> &'static str {
        self.subscriber.get_name()
    }

    fn subscribe_to<P, Input>(&mut self, publisher: &mut P) -> Result<()>
    where
        P: PublisherWrapper<Input, Self::InputMessage>,
        Input: Send + 'static,
    {
        let publisher_name = PublisherWrapper::get_name(publisher);
        self.publisher_name = Some(publisher_name);
        log::info!("({}) <-> ({})", self.subscriber.get_name(), publisher_name,);
        self.subscriber.subscribe_to(publisher)
    }

    fn receive(&mut self) -> BoxFuture<'_, Self::OutputMessage> {
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
        .boxed()
    }
}
