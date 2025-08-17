use futures::{Stream, future::BoxFuture};
use std::ops::{Deref, DerefMut};
use std::pin::Pin;

use crate::Result;

/// A trait for types that can publish messages to subscribers.
///
/// This trait defines the core functionality for a publisher in a publisher-subscriber pattern.
pub trait Publisher {
    /// The type of messages that this publisher can send.
    /// Must implement Send and have a static lifetime.
    type InputMessage: Send + 'static;

    /// The type of message that the subscriber of this publisher
    /// will receive.
    /// Must implement Send and have a static lifetime.
    type OutputMessage: Send + 'static;

    /// Returns the name of the publisher.
    ///
    /// # Returns
    /// A string slice containing the publisher's name.
    fn get_name(&self) -> &'static str;

    /// Publishes a message to all subscribers.
    ///
    /// # Arguments
    /// * `message` - The message to publish
    ///
    /// # Returns
    /// A future that resolves to a Result indicating success or failure of the publish operation.
    fn publish(&self, message: Self::InputMessage) -> BoxFuture<'_, Result<()>>;

    /// Creates a new message stream for a subscriber.
    ///
    /// # Arguments
    /// * `subscriber_name` - The name of the subscriber requesting the stream
    ///
    /// # Returns
    /// A Result containing a pinned box with the message stream if successful.
    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<Pin<Box<dyn Stream<Item = Self::OutputMessage> + Send + Sync + 'static>>>;
}

// Add blanket implementation for types that can be dereferenced into a Publisher
impl<T> Publisher for T
where
    T: Deref + DerefMut,
    T::Target: Publisher,
{
    type InputMessage = <T::Target as Publisher>::InputMessage;
    type OutputMessage = <T::Target as Publisher>::OutputMessage;

    fn get_name(&self) -> &'static str {
        self.deref().get_name()
    }

    fn publish(&self, message: Self::InputMessage) -> BoxFuture<'_, Result<()>> {
        self.deref().publish(message)
    }

    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<Pin<Box<dyn Stream<Item = Self::OutputMessage> + Send + Sync + 'static>>> {
        self.deref_mut().get_message_stream(subscriber_name)
    }
}

/// A trait for types that wrap a publisher.
///
/// This trait provides a default implementation for publisher operations
/// by delegating to an internal publisher instance.
pub trait PublisherWrapper<InputMessage, OutputMessage>
where
    InputMessage: Send + 'static,
    OutputMessage: Send + 'static,
{
    /// Gets a reference to the wrapped publisher.
    ///
    /// # Returns
    /// A reference to the wrapped publisher implementation.
    fn get_publisher(
        &self,
    ) -> &impl Publisher<InputMessage = InputMessage, OutputMessage = OutputMessage>;

    /// Gets a mutable reference to the wrapped publisher.
    ///
    /// # Returns
    /// A mutable reference to the wrapped publisher implementation.
    fn get_publisher_mut(
        &mut self,
    ) -> &mut impl Publisher<InputMessage = InputMessage, OutputMessage = OutputMessage>;

    /// Returns the name of the wrapped publisher.
    ///
    /// # Returns
    /// A static string slice containing the publisher's name.
    fn get_name(&self) -> &'static str {
        Publisher::get_name(self.get_publisher())
    }

    /// Publishes a message using the wrapped publisher.
    ///
    /// # Arguments
    /// * `message` - The message to publish
    ///
    /// # Returns
    /// A future that resolves to a Result indicating success or failure of the publish operation.
    fn publish(&self, message: InputMessage) -> futures::future::BoxFuture<'_, Result<()>> {
        Publisher::publish(self.get_publisher(), message)
    }

    /// Creates a new message stream using the wrapped publisher.
    ///
    /// # Arguments
    /// * `subscriber_name` - The name of the subscriber requesting the stream
    ///
    /// # Returns
    /// A Result containing a pinned box with the message stream if successful.
    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<std::pin::Pin<Box<dyn futures::Stream<Item = OutputMessage> + Send + Sync + 'static>>>
    {
        Publisher::get_message_stream(self.get_publisher_mut(), subscriber_name)
    }
}

impl<T> PublisherWrapper<T::InputMessage, T::OutputMessage> for T
where
    T: Publisher,
{
    fn get_publisher(
        &self,
    ) -> &impl Publisher<InputMessage = T::InputMessage, OutputMessage = T::OutputMessage> {
        self
    }

    fn get_publisher_mut(
        &mut self,
    ) -> &mut impl Publisher<InputMessage = T::InputMessage, OutputMessage = T::OutputMessage> {
        self
    }
}
