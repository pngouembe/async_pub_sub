use crate::{PublisherWrapper, Result, Subscriber};
use futures::{stream::SelectAll, Stream, StreamExt};
use std::{future::Future, pin::Pin};

/// A concrete implementation of the Subscriber trait that can receive messages from multiple publishers.
///
/// This implementation uses a SelectAll stream to merge messages from multiple publishers
/// into a single stream that can be read from sequentially.
pub struct SubscriberImpl<Message>
where
    Message: Send + 'static,
{
    /// The unique name identifier of this subscriber
    name: &'static str,
    /// Combined stream of messages from all subscribed publishers
    messages: SelectAll<Pin<Box<dyn Stream<Item = Message> + Send + Sync + 'static>>>,
}

impl<Message> SubscriberImpl<Message>
where
    Message: Send + 'static,
{
    /// Creates a new SubscriberImpl with the given name.
    ///
    /// # Parameters
    /// * `name` - A static string that uniquely identifies this subscriber
    ///
    /// # Returns
    /// A new instance of SubscriberImpl
    pub fn new(name: &'static str) -> Self {
        let messages = SelectAll::new();
        Self { name, messages }
    }

    /// Subscribes to a publisher to receive its messages.
    ///
    /// # Parameters
    /// * `publisher` - A mutable reference to any type implementing PublisherWrapper
    ///
    /// # Returns
    /// A Result indicating success or failure of the subscription
    pub fn subscribe_to(&mut self, publisher: &mut impl PublisherWrapper<Message>) -> Result<()> {
        let stream = publisher.get_message_stream(self.name)?;
        self.messages.push(stream);
        Ok(())
    }

    /// Asynchronously receives the next available message from any subscribed publisher.
    ///
    /// # Returns
    /// The next message in the combined message stream
    pub async fn receive(&mut self) -> Message {
        self.messages.select_next_some().await
    }
}

/// Implementation of the Subscriber trait for SubscriberImpl
///
/// This provides the standard subscriber interface by delegating to the
/// implementation-specific methods.
impl<Message> Subscriber for SubscriberImpl<Message>
where
    Message: Send + 'static,
{
    type Message = Message;

    fn get_name(&self) -> &'static str {
        self.name
    }

    fn subscribe_to(&mut self, publisher: &mut impl PublisherWrapper<Self::Message>) -> Result<()> {
        SubscriberImpl::subscribe_to(self, publisher)
    }

    fn receive(&mut self) -> impl Future<Output = Message> {
        SubscriberImpl::receive(self)
    }
}
