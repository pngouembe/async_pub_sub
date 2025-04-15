use std::fmt::Debug;

use async_pub_sub::{Publisher, PublisherImpl, Result, Subscriber, SubscriberImpl};
use futures::{FutureExt, future::BoxFuture};

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

impl<Message> Subscriber for LoggingSubscriber<SubscriberImpl<Message>>
where
    Message: Debug + Send + 'static,
{
    type Message = Message;
    fn get_name(&self) -> &'static str {
        self.subscriber.get_name()
    }

    fn subscribe_to(&mut self, publisher: &mut dyn Publisher<Message = Message>) -> Result<()> {
        let publisher_name = Publisher::get_name(publisher);
        self.subscriber.subscribe_to(publisher)?;
        self.publisher_name = Some(publisher_name);
        log::info!("({}) <-> ({})", self.subscriber.get_name(), publisher_name,);
        Ok(())
    }

    fn receive(&mut self) -> BoxFuture<Message> {
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
        .boxed()
    }
}

#[test_log::test(tokio::test)]
async fn test_logging_subscriber() -> Result<()> {
    // -- Setup & Fixtures
    let mut subscriber = LoggingSubscriber::new(SubscriberImpl::new("subscriber"));
    let mut publisher = PublisherImpl::new("publisher", 10);

    subscriber.subscribe_to(&mut publisher)?;

    // -- Exec
    publisher.publish(42).await?;
    let message = subscriber.receive().await;

    // -- Check
    assert_eq!(message, 42);

    Ok(())
}
