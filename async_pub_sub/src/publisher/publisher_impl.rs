use futures::{FutureExt, SinkExt};

use super::Publisher;
use crate::Result;

/// A concrete implementation of the Publisher trait that handles message distribution
/// to a single subscriber.
///
/// # Type Parameters
///
/// * `Message` - The type of message that can be published. Must implement `Send` and have
///   a static lifetime.
///
/// # Fields
///
/// * `name` - A static identifier for the publisher
/// * `subscriber_name` - The name of the currently bound subscriber, if any
/// * `sender` - The sending half of the message channel
/// * `receiver` - The receiving half of the channel, available until a subscriber is bound
///
/// # Example
///
/// ```
/// use async_pub_sub::PublisherImpl;
/// use async_pub_sub::SubscriberImpl;
///
/// #[tokio::main]
/// async fn main() {
///     let mut publisher = PublisherImpl::<String>::new("my_publisher", 10);
///     let mut subscriber = SubscriberImpl::new("subscriber");
///     
///     subscriber.subscribe_to(&mut publisher).unwrap();
///     
///     // Publish a message
///     publisher.publish("Hello, World!".to_string()).await.unwrap();
///     
///     // Receive the message
///     let message = subscriber.receive().await;
///     assert_eq!(message, "Hello, World!");
/// }
/// ```
pub struct PublisherImpl<Message>
where
    Message: Send + 'static,
{
    name: &'static str,
    subscriber_name: Option<&'static str>,
    sender: futures::channel::mpsc::Sender<Message>,
    receiver: Option<futures::channel::mpsc::Receiver<Message>>,
}

impl<Message> PublisherImpl<Message>
where
    Message: Send + 'static,
{
    /// Creates a new publisher with the specified name and message buffer capacity.
    ///
    /// # Arguments
    ///
    /// * `name` - A static string identifier for the publisher
    /// * `buffer_size` - The size of the message buffer for the underlying channel
    ///
    /// # Returns
    ///
    /// A new `PublisherImpl` instance
    pub fn new(name: &'static str, buffer_size: usize) -> Self {
        let (sender, receiver) = futures::channel::mpsc::channel(buffer_size);
        Self {
            name,
            subscriber_name: None,
            sender,
            receiver: Some(receiver),
        }
    }

    /// Publishes a message to the channel.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to publish
    ///
    /// # Returns
    ///
    /// A Result indicating whether the message was successfully sent
    pub async fn publish(&self, message: Message) -> Result<()> {
        self.sender.clone().send(message).await?;
        Ok(())
    }
}

impl<Message> Publisher for PublisherImpl<Message>
where
    Message: Send + Sync + 'static,
{
    type Message = Message;

    /// Returns the name of the publisher.
    fn get_name(&self) -> &'static str {
        self.name
    }

    /// Publishes a message through the channel.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to publish
    ///
    /// # Returns
    ///
    /// A boxed future that resolves to a Result indicating success or failure
    fn publish(&self, message: Self::Message) -> futures::future::BoxFuture<Result<()>> {
        let mut sender = self.sender.clone();
        async move {
            sender
                .send(message)
                .await
                .map_err(|err| format!("Failed to send message (err: {err})").into())
        }
        .boxed()
    }

    /// Binds a subscriber to this publisher and returns the message stream.
    ///
    /// # Arguments
    ///
    /// * `subscriber_name` - The name of the subscriber to bind
    ///
    /// # Returns
    ///
    /// A Result containing either the message stream or an error if a subscriber
    /// is already bound
    ///
    /// # Errors
    ///
    /// Returns an error if the publisher is already bound to another subscriber
    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<std::pin::Pin<Box<dyn futures::Stream<Item = Self::Message> + Send + Sync + 'static>>>
    {
        let Some(receiver) = self.receiver.take() else {
            return Err(format!(
                "{} publisher can only be bound to one subscriber (already bound to {})",
                self.name,
                self.subscriber_name
                    .expect("the subscriber name should be known at this point")
            )
            .into());
        };

        self.subscriber_name = Some(subscriber_name);

        Ok(Box::pin(receiver))
    }
}
