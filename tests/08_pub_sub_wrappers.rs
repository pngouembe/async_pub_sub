use std::pin::Pin;

use futures::Stream;
use tokio_pub_sub::{LoggingPublisher, Publisher, Result, SimpleSubscriber, Subscriber};

struct Service {
    subscriber: SimpleSubscriber<i32>,
    publisher: LoggingPublisher<String>,
}

impl Service {
    pub fn new() -> Self {
        let subscriber = SimpleSubscriber::new("Service");
        let publisher = LoggingPublisher::new("Service", 10);

        Self {
            subscriber,
            publisher,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            let message = self.subscriber.receive().await;
            self.publisher.publish_event(message.to_string()).await?;
        }
    }
}

impl Subscriber<i32> for Service {
    fn get_name(&self) -> &'static str {
        self.subscriber.get_name()
    }

    fn subscribe_to(&mut self, publisher: &mut impl Publisher<i32>) -> Result<()> {
        self.subscriber.subscribe_to(publisher)
    }

    fn receive(&mut self) -> impl std::future::Future<Output = i32> {
        self.subscriber.receive()
    }
}

impl Publisher<String> for Service {
    fn get_name(&self) -> &'static str {
        self.publisher.get_name()
    }

    fn publish_event(&self, message: String) -> futures::future::BoxFuture<Result<()>> {
        self.publisher.publish_event(message)
    }

    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<Pin<Box<dyn Stream<Item = String> + Send + Sync + 'static>>> {
        self.publisher.get_message_stream(subscriber_name)
    }
}

#[test_log::test(tokio::test)]
async fn test_pub_sub_wrapper() -> Result<()> {
    // -- Setup & Fixtures
    let mut publisher = LoggingPublisher::new("publisher", 1);
    let mut subscriber = SimpleSubscriber::new("subscriber");
    let mut service = Service::new();

    service.subscribe_to(&mut publisher)?;
    subscriber.subscribe_to(&mut service)?;

    tokio::spawn(async move {
        service.run().await.unwrap();
    });

    // -- Exec
    publisher.publish_event(42).await?;

    let message = subscriber.receive().await;

    // -- Check
    assert_eq!(message, 42.to_string());

    Ok(())
}
