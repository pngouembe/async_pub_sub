use std::{fmt::Display, pin::Pin};

use futures::{future::BoxFuture, FutureExt, Stream};

use crate::{Publisher, PublisherLayer, Result};

/// A publisher middleware layer that adds logging capabilities to any publisher.
/// This layer will log all messages that are published through the publisher.
pub struct LoggingPublisherLayer;

/// Implementation of the PublisherLayer trait for LoggingPublisherLayer.
/// This allows wrapping any publisher that implements the Publisher trait with logging functionality.
impl<P> PublisherLayer<P> for LoggingPublisherLayer
where
    P: Publisher + Send + Sync,
    P::Message: Display,
{
    type PublisherType = LoggingPublisher<P>;

    /// Creates a new LoggingPublisher by wrapping the provided publisher.
    fn layer(&self, publisher: P) -> Self::PublisherType {
        LoggingPublisher {
            subscriber_name: None,
            publisher,
        }
    }
}

/// A publisher wrapper that adds logging functionality to an existing publisher.
/// Logs all messages that are published through this publisher.
pub struct LoggingPublisher<P>
where
    P: Publisher,
{
    /// The name of the subscriber receiving messages, set when get_message_stream is called
    subscriber_name: Option<&'static str>,
    /// The underlying publisher being wrapped
    publisher: P,
}

/// Implementation of the Publisher trait for LoggingPublisher.
/// This implementation delegates all operations to the wrapped publisher while adding logging.
impl<P> Publisher for LoggingPublisher<P>
where
    P: Publisher,
    P::Message: Display,
    Self: Sync,
{
    type Message = P::Message;

    /// Returns the name of the underlying publisher
    fn get_name(&self) -> &'static str {
        self.publisher.get_name()
    }

    /// Publishes a message and logs the operation with source publisher and destination subscriber
    fn publish(&self, message: Self::Message) -> BoxFuture<Result<()>> {
        async move {
            let message_str = format!("{}", &message);
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

    /// Sets up a message stream for a subscriber and stores the subscriber's name for logging
    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<Pin<Box<dyn Stream<Item = Self::Message> + Send + Sync + 'static>>> {
        self.subscriber_name = Some(subscriber_name);
        self.publisher.get_message_stream(subscriber_name)
    }
}
