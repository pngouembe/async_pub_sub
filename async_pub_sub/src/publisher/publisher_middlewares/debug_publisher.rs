use std::{fmt::Debug, pin::Pin};

use futures::{future::BoxFuture, FutureExt, Stream};

use crate::{Layer, Publisher, Result};

/// A middleware layer that adds debug logging capabilities to a publisher.
/// When messages are published, it will log them using the debug format.
pub struct DebuggingPublisherLayer;

impl<P> Layer<P> for DebuggingPublisherLayer
where
    P: Publisher + Send,
    P::Message: Debug,
{
    type LayerType = DebugPublisher<P>;

    fn layer(&self, publisher: P) -> Self::LayerType {
        DebugPublisher {
            subscriber_name: None,
            publisher,
        }
    }
}

/// A publisher wrapper that adds debug logging capabilities to an existing publisher.
/// It logs messages when they are published, showing the publisher name, subscriber name,
/// and the debug representation of the message.
pub struct DebugPublisher<P>
where
    P: Publisher,
{
    /// The name of the subscriber receiving messages, set when get_message_stream is called
    subscriber_name: Option<&'static str>,
    /// The underlying publisher being wrapped
    publisher: P,
}

impl<P> Publisher for DebugPublisher<P>
where
    P: Publisher,
    P::Message: Debug,
    Self: Sync,
{
    type Message = P::Message;

    /// Returns the name of the underlying publisher
    fn get_name(&self) -> &'static str {
        self.publisher.get_name()
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
            let result = self.publisher.publish(message).await;
            log::info!(
                "[{}] -> [{}]: {}",
                self.publisher.get_name(),
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
        self.publisher.get_message_stream(subscriber_name)
    }
}
