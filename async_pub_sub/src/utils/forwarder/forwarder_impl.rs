use std::pin::Pin;

use crate::{Publisher, Result, Subscriber, SubscriberImpl};
use futures::{FutureExt, Stream, future::BoxFuture, stream};

use super::Forwarder;

/// A middleware component that logs messages as they pass between a publisher and subscriber.
///
/// The forwarder acts as both a subscriber (to receive messages) and a publisher (to forward messages).
/// It logs each message that passes through it, providing visibility into the message flow.
///
/// # Type Parameters
/// * `Message` - The type of message being forwarded. Must be `Send`.
pub struct ForwarderImpl<Message>
where
    Message: Send + 'static,
{
    /// The name identifier for this forwarder instance
    name: &'static str,
    /// The name of the subscriber this forwarder is connected to, if any
    subscriber_name: Option<&'static str>,
    /// The internal subscriber implementation used to receive messages
    subscriber: Option<SubscriberImpl<Message>>,
}

impl<Message> ForwarderImpl<Message>
where
    Message: Send + 'static,
{
    /// Creates a new LoggingForwarder with the specified name.
    ///
    /// # Arguments
    /// * `name` - A static string identifier for this forwarder instance
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            subscriber_name: None,
            subscriber: Some(SubscriberImpl::new(name)),
        }
    }
}

impl<Message> Subscriber for ForwarderImpl<Message>
where
    Message: Send + 'static,
{
    type Message = Message;

    /// Returns the name of this forwarder instance.
    fn get_name(&self) -> &'static str {
        self.name
    }

    /// Subscribes this forwarder to a publisher.
    ///
    /// # Arguments
    /// * `publisher` - The publisher to subscribe to
    ///
    /// # Returns
    /// * `Result<()>` - Ok if subscription successful, Err with description if failed
    fn subscribe_to(&mut self, publisher: &mut dyn Publisher<Message = Message>) -> Result<()> {
        let Some(subscriber) = self.subscriber.as_mut() else {
            let subscriber_name = self
                .subscriber_name
                .expect("the subscriber name should be known at this point");

            return Err(format!(
                "{} forwarder has already been bound to {}, subscribe to {} before {} subscribes to it",
                self.name,
                subscriber_name,
                self.name,
                subscriber_name
            )
            .into());
        };

        subscriber.subscribe_to(publisher)
    }

    /// Not implemented for LoggingForwarder. Will panic if called.
    fn receive(&mut self) -> BoxFuture<Message> {
        panic!("LoggingForwarder does not implement receive method")
    }
}

impl<Message> Publisher for ForwarderImpl<Message>
where
    Message: Send + 'static,
{
    type Message = Message;

    /// Returns the name of this forwarder instance.
    fn get_name(&self) -> &'static str {
        self.name
    }

    /// Not implemented for LoggingForwarder. Will panic if called.
    fn publish(&self, _message: Message) -> futures::future::BoxFuture<Result<()>> {
        async move { panic!("LoggingForwarder does not implement publish method") }.boxed()
    }

    /// Creates a message stream that logs messages as they pass through.
    ///
    /// # Arguments
    /// * `subscriber_name` - The name of the subscriber that will receive messages
    ///
    /// # Returns
    /// * `Result<Pin<Box<dyn Stream<Item = Message>>>>` - A stream of messages if successful,
    ///   Err with description if the forwarder is already bound to another subscriber
    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<Pin<Box<dyn Stream<Item = Message> + Send + Sync + 'static>>> {
        let Some(subscriber) = self.subscriber.take() else {
            return Err(format!(
                "{} forwarder can only be bound to one subscriber (already bound to {})",
                self.name,
                self.subscriber_name
                    .expect("the subscriber name should be known at this point")
            )
            .into());
        };
        self.subscriber_name = Some(subscriber_name);

        let stream = Box::pin(stream::unfold(
            subscriber,
            move |mut subscriber| async move {
                let message = subscriber.receive().await;
                Some((message, subscriber))
            },
        ));

        Ok(stream)
    }
}

impl<Message> Forwarder for ForwarderImpl<Message> where Message: Send + 'static {}
