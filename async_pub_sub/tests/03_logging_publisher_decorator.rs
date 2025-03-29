use std::{fmt::Debug, pin::Pin};

use async_pub_sub::{Publisher, PublisherImpl, Result, SubscriberImpl};
use futures::{future::BoxFuture, FutureExt, Stream};

struct LoggingPublisher<P> {
    subscriber_name: Option<&'static str>,
    publisher: P,
}

impl<P> LoggingPublisher<P> {
    pub fn new(publisher: P) -> Self {
        Self {
            subscriber_name: None,
            publisher,
        }
    }
}

impl<Message> Publisher for LoggingPublisher<PublisherImpl<Message>>
where
    Message: Debug + Send + Sync + 'static,
{
    type Message = Message;

    fn get_name(&self) -> &'static str {
        self.publisher.get_name()
    }

    fn publish(&self, message: Message) -> BoxFuture<Result<()>> {
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

    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<Pin<Box<dyn Stream<Item = Message> + Send + Sync + 'static>>> {
        let stream = self.publisher.get_message_stream(subscriber_name)?;
        self.subscriber_name = Some(subscriber_name);
        log::info!("({}) <-> ({})", self.publisher.get_name(), subscriber_name);
        Ok(stream)
    }
}

#[test_log::test(tokio::test)]
async fn test_logging_publisher() -> Result<()> {
    // -- Setup & Fixtures
    let mut subscriber = SubscriberImpl::new("subscriber");
    let mut publisher = LoggingPublisher::new(PublisherImpl::new("publisher", 10));

    subscriber.subscribe_to(&mut publisher)?;

    // -- Exec
    publisher.publish(42).await?;
    let message = subscriber.receive().await;

    // -- Check
    assert_eq!(message, 42);

    Ok(())
}
