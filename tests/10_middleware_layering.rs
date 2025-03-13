use std::{fmt::Display, pin::Pin};

use futures::{future::BoxFuture, FutureExt, Stream};
use tokio_pub_sub::{Publisher, PublisherLayer, Result, SimpleSubscriber};

struct LoggingPublisherLayer;

impl<P, Message> PublisherLayer<P, Message> for LoggingPublisherLayer
where
    P: Publisher<Message> + Send + Sync,
    Message: Display + Send + Sync + 'static,
{
    type PublisherType = LoggingPublisher<P, Message>;

    fn layer(&self, publisher: P) -> Self::PublisherType {
        LoggingPublisher {
            subscriber_name: None,
            publisher,
            _phantom: std::marker::PhantomData,
        }
    }
}

struct LoggingPublisher<P, Message>
where
    P: Publisher<Message>,
    Message: Display + Send + 'static,
{
    subscriber_name: Option<&'static str>,
    publisher: P,
    _phantom: std::marker::PhantomData<Message>,
}

impl<P, Message> Publisher<Message> for LoggingPublisher<P, Message>
where
    P: Publisher<Message>,
    Message: Display + Send + 'static,
    Self: Sync,
{
    fn get_name(&self) -> &'static str {
        self.publisher.get_name()
    }

    fn publish_event(&self, message: Message) -> BoxFuture<tokio_pub_sub::Result<()>> {
        async move {
            let message_str = format!("{}", &message);
            let result = self.publisher.publish_event(message).await;
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

    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<Pin<Box<dyn Stream<Item = Message> + Send + Sync + 'static>>> {
        self.subscriber_name = Some(subscriber_name);
        self.publisher.get_message_stream(subscriber_name)
    }
}

#[test_log::test(tokio::test)]
async fn test_middleware_wrapping() -> Result<()> {
    // -- Setup & Fixtures
    let publisher = tokio_pub_sub::SimplePublisher::new("publisher", 10);
    let mut logging_publisher = LoggingPublisherLayer.layer(publisher);

    let mut subscriber = SimpleSubscriber::new("subscriber");
    subscriber.subscribe_to(&mut logging_publisher)?;

    // -- Exercise
    let message = "Hello, World!";
    logging_publisher.publish_event(message.to_string()).await?;

    // -- Verification
    Ok(())
}
