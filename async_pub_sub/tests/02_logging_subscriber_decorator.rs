use std::fmt::Debug;

use async_pub_sub::{MultiPublisher, Result, Subscriber};
use tokio_implementations::{publisher::mpsc::MpscPublisher, subscriber::mpsc::MpscSubscriber};

struct LoggingSubscriber<S> {
    publisher_name: Option<&'static str>,
    subscriber: S,
}

impl<S> LoggingSubscriber<S> {
    pub fn new(subscriber: S) -> Self {
        Self {
            publisher_name: None,
            subscriber,
        }
    }
}

impl<Message> Subscriber for LoggingSubscriber<MpscSubscriber<Message>>
where
    Message: Debug + Send + 'static,
{
    type Message = Message;
    fn get_name(&self) -> &'static str {
        self.subscriber.get_name()
    }

    fn subscribe_to(&mut self, publisher: &mut impl MultiPublisher<Self::Message>) -> Result<()> {
        self.subscriber.subscribe_to(publisher)?;
        self.publisher_name = Some(publisher.get_name());
        log::info!(
            "({}) <-> ({})",
            self.subscriber.get_name(),
            publisher.get_name(),
        );
        Ok(())
    }

    fn receive(&mut self) -> impl std::future::Future<Output = Message> {
        let publisher_name = self.publisher_name.expect("publisher name should be known");
        async move {
            let message = self.subscriber.receive().await;
            log::info!(
                "[{}] <- [{}]: {:?}",
                self.subscriber.get_name(),
                publisher_name,
                message
            );
            message
        }
    }
}

#[test_log::test(tokio::test)]
async fn test_logging_subscriber() -> Result<()> {
    // -- Setup & Fixtures
    let mut subscriber = LoggingSubscriber::new(MpscSubscriber::new("subscriber"));
    let mut publisher = MpscPublisher::new("publisher", 10);

    subscriber.subscribe_to(&mut publisher)?;

    // -- Exec
    publisher.publish(42).await?;
    let message = subscriber.receive().await;

    // -- Check
    assert_eq!(message, 42);

    Ok(())
}
