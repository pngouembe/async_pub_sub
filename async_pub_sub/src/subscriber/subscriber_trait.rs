use futures::future::BoxFuture;
use std::future::Future;
use std::ops::{Deref, DerefMut};

use crate::{PublisherWrapper, Result};

/// Defines the core functionality for a subscriber in the pub/sub system.
///
/// A subscriber can subscribe to publishers and receive messages from them.
pub trait Subscriber {
    /// The type of message this subscriber can receive.
    /// Must implement Send and have static lifetime.
    type InputMessage: Send + 'static;

    /// The type of message this subscriber will output when calling
    /// the receive message.
    /// Must implement Send and have static lifetime.
    type OutputMessage: Send + 'static;

    /// Returns the unique name identifier of the subscriber.
    ///
    /// # Returns
    /// A static string slice containing the subscriber's name.
    fn get_name(&self) -> &'static str;

    /// Subscribes this subscriber to a publisher.
    ///
    /// # Parameters
    /// * `publisher` - A mutable reference to any type implementing PublisherWrapper
    ///
    /// # Returns
    /// A Result indicating success or failure of the subscription
    fn subscribe_to<P, Input>(&mut self, publisher: &mut P) -> Result<()>
    where
        P: PublisherWrapper<Input, Self::InputMessage>,
        Input: Send + 'static;

    /// Asynchronously receives the next message from subscribed publishers.
    ///
    /// # Returns
    /// A Future that resolves to the next message of type Message
    fn receive(&mut self) -> BoxFuture<'_, Self::OutputMessage>;
}

// Add blanket implementation for types that can be dereferenced into a Subscriber
impl<T> Subscriber for T
where
    T: Deref + DerefMut,
    T::Target: Subscriber,
{
    type InputMessage = <T::Target as Subscriber>::InputMessage;
    type OutputMessage = <T::Target as Subscriber>::OutputMessage;

    fn get_name(&self) -> &'static str {
        self.deref().get_name()
    }

    fn subscribe_to<P, Input>(&mut self, publisher: &mut P) -> Result<()>
    where
        P: PublisherWrapper<Input, Self::InputMessage>,
        Input: Send + 'static,
    {
        self.deref_mut().subscribe_to(publisher)
    }

    fn receive(&mut self) -> BoxFuture<'_, Self::OutputMessage> {
        self.deref_mut().receive()
    }
}

/// A wrapper trait that provides a unified interface for working with Subscriber implementations.
///
/// This trait allows for generic handling of different subscriber types while maintaining
/// type safety for the message type.
pub trait SubscriberWrapper<InputMessage, OutputMessage>
where
    InputMessage: Send + 'static,
    OutputMessage: Send + 'static,
{
    /// Gets an immutable reference to the underlying subscriber implementation.
    fn get_subscriber(
        &self,
    ) -> &impl Subscriber<InputMessage = InputMessage, OutputMessage = OutputMessage>;

    /// Gets a mutable reference to the underlying subscriber implementation.
    fn get_subscriber_mut(
        &mut self,
    ) -> &mut impl Subscriber<InputMessage = InputMessage, OutputMessage = OutputMessage>;

    /// Returns the name of the subscriber.
    /// Delegates to the underlying subscriber's get_name implementation.
    fn get_name(&self) -> &'static str {
        Subscriber::get_name(self.get_subscriber())
    }

    /// Subscribes to a publisher.
    /// Delegates to the underlying subscriber's subscribe_to implementation.
    fn subscribe_to<P, Input>(&mut self, publisher: &mut P) -> Result<()>
    where
        P: PublisherWrapper<Input, InputMessage>,
        Input: Send + 'static,
    {
        Subscriber::subscribe_to(self.get_subscriber_mut(), publisher)
    }

    /// Receives the next message asynchronously.
    /// Delegates to the underlying subscriber's receive implementation.
    fn receive(&mut self) -> impl Future<Output = OutputMessage> {
        Subscriber::receive(self.get_subscriber_mut())
    }
}

/// Blanket implementation of SubscriberWrapper for any type that implements Subscriber.
///
/// This allows any Subscriber implementation to automatically gain the wrapper functionality.
impl<T> SubscriberWrapper<T::InputMessage, T::OutputMessage> for T
where
    T: Subscriber,
{
    fn get_subscriber(
        &self,
    ) -> &impl Subscriber<InputMessage = T::InputMessage, OutputMessage = T::OutputMessage> {
        self
    }

    fn get_subscriber_mut(
        &mut self,
    ) -> &mut impl Subscriber<InputMessage = T::InputMessage, OutputMessage = T::OutputMessage>
    {
        self
    }
}
