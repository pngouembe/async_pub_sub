use std::{fmt::Debug, pin::Pin};

use futures::{FutureExt, Stream, StreamExt, future::BoxFuture};

use crate::{Layer, Publisher, Result, Subscriber, utils::forwarder::forwarder_trait::Forwarder};

/// A middleware layer that adds debug logging capabilities to a publisher.
/// When messages are published, it will log them using the debug format.
pub struct DebuggingForwarderLayer;

impl<F> Layer<F> for DebuggingForwarderLayer
where
    F: Forwarder + Send,
    <F as Subscriber>::Message: Debug,
    <F as Publisher>::Message: Debug,
{
    type LayerType = DebugForwarder<F>;

    fn layer(&self, forwarder: F) -> Self::LayerType {
        DebugForwarder {
            subscriber_name: None,
            forwarder,
        }
    }
}

/// A publisher wrapper that adds debug logging capabilities to an existing publisher.
/// It logs messages when they are published, showing the publisher name, subscriber name,
/// and the debug representation of the message.
pub struct DebugForwarder<F>
where
    F: Forwarder,
{
    /// The name of the subscriber receiving messages, set when get_message_stream is called
    subscriber_name: Option<&'static str>,
    /// The underlying publisher being wrapped
    forwarder: F,
}

impl<F> Publisher for DebugForwarder<F>
where
    F: Forwarder,
    <F as Publisher>::Message: Debug,
    Self: Sync,
{
    type Message = <F as Publisher>::Message;

    /// Returns the name of the underlying publisher
    fn get_name(&self) -> &'static str {
        Publisher::get_name(&self.forwarder)
    }

    /// Publishes a message while logging its debug representation.
    ///
    /// # Arguments
    /// * `message` - The message to publish
    ///
    /// Logs the message in the format: "[publisher_name] -> [subscriber_name]: message_debug_format"
    fn publish(&self, message: Self::Message) -> BoxFuture<Result<()>> {
        async move {
            let message_str = format!("{:?}", &message);
            let result = self.forwarder.publish(message).await;
            log::info!(
                "[{}] -> [{}]: {}",
                Publisher::get_name(&self.forwarder),
                self.subscriber_name
                    .expect("subscriber name should be known"),
                message_str
            );
            result
        }
        .boxed()
    }

    /// Gets a message stream for the given subscriber name.
    /// Stores the subscriber name for use in debug logging.
    ///
    /// # Arguments
    /// * `subscriber_name` - The name of the subscriber requesting the stream
    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<Pin<Box<dyn Stream<Item = Self::Message> + Send + Sync + 'static>>> {
        self.subscriber_name = Some(subscriber_name);

        let publisher_name = Publisher::get_name(&self);

        self.forwarder
            .get_message_stream(subscriber_name)
            .map(|stream| {
                let stream: Pin<Box<dyn Stream<Item = Self::Message> + Send + Sync + 'static>> =
                    Box::pin(stream.map(move |message| {
                        log::info!(
                            "[{}] -> [{}]: {:?}",
                            publisher_name,
                            subscriber_name,
                            message
                        );
                        message
                    }));

                stream
            })
    }
}

impl<F> Subscriber for DebugForwarder<F>
where
    F: Forwarder,
{
    type Message = <F as Subscriber>::Message;

    fn get_name(&self) -> &'static str {
        Subscriber::get_name(&self.forwarder)
    }

    fn subscribe_to(
        &mut self,
        publisher: &mut dyn Publisher<Message = Self::Message>,
    ) -> Result<()> {
        self.forwarder.subscribe_to(publisher)
    }

    fn receive(&mut self) -> BoxFuture<Self::Message> {
        self.forwarder.receive()
    }
}

impl<F> Forwarder for DebugForwarder<F>
where
    F: Forwarder + Sync,
    <F as Publisher>::Message: Debug,
    <F as Subscriber>::Message: Debug + Send + Sync + 'static,
{
}
