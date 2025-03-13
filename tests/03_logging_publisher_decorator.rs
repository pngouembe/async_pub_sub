use std::{fmt::Debug, pin::Pin};

use futures::{future::BoxFuture, FutureExt, Stream};
use tokio_pub_sub::{Publisher, Result, SimplePublisher, SimpleSubscriber};

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

impl<Message> Publisher<Message> for LoggingPublisher<SimplePublisher<Message>>
where
    Message: Debug + Send + 'static,
{
    fn get_name(&self) -> &'static str {
        self.publisher.get_name()
    }

    fn publish_event(&self, message: Message) -> BoxFuture<Result<()>> {
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
    let mut subscriber = SimpleSubscriber::new("subscriber");
    let mut publisher = LoggingPublisher::new(SimplePublisher::new("publisher", 10));

    subscriber.subscribe_to(&mut publisher)?;

    // -- Exec
    publisher.publish_event(42).await?;
    let message = subscriber.receive().await;

    // -- Check
    assert_eq!(message, 42);

    Ok(())
}
