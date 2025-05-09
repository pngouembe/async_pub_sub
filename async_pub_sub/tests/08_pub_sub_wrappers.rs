// TODO: rework the test

use std::pin::Pin;

use async_pub_sub::{Publisher, PublisherImpl, Result, Subscriber, SubscriberImpl};
use futures::{FutureExt, Stream, future::BoxFuture};

struct Service {
    subscriber: SubscriberImpl<i32>,
    publisher: PublisherImpl<String>,
}

impl Service {
    pub fn new() -> Self {
        let subscriber = SubscriberImpl::new("Service");
        let publisher = PublisherImpl::new("Service", 10);

        Self {
            subscriber,
            publisher,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            let message = self.subscriber.receive().await;
            Publisher::publish(&self.publisher, message.to_string()).await?;
        }
    }
}

impl Subscriber for Service {
    type Message = i32;

    fn get_name(&self) -> &'static str {
        self.subscriber.get_name()
    }

    fn subscribe_to(
        &mut self,
        publisher: &mut dyn Publisher<Message = Self::Message>,
    ) -> Result<()> {
        self.subscriber.subscribe_to(publisher)
    }

    fn receive(&mut self) -> BoxFuture<Self::Message> {
        self.subscriber.receive().boxed()
    }
}

impl Publisher for Service {
    type Message = String;

    fn get_name(&self) -> &'static str {
        Publisher::get_name(&self.publisher)
    }

    fn publish(&self, message: String) -> futures::future::BoxFuture<Result<()>> {
        Publisher::publish(&self.publisher, message)
    }

    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<Pin<Box<dyn Stream<Item = String> + Send + Sync + 'static>>> {
        Publisher::get_message_stream(&mut self.publisher, subscriber_name)
    }
}

#[test_log::test(tokio::test)]
async fn test_pub_sub_wrapper() -> Result<()> {
    // -- Setup & Fixtures
    let mut publisher = PublisherImpl::new("publisher", 1);
    let mut subscriber = SubscriberImpl::new("subscriber");
    let mut service = Service::new();

    service.subscribe_to(&mut publisher)?;
    subscriber.subscribe_to(&mut service)?;

    tokio::spawn(async move {
        service.run().await.unwrap();
    });

    // -- Exec
    Publisher::publish(&publisher, 42).await?;

    let message = subscriber.receive().await;

    // -- Check
    assert_eq!(message, 42.to_string());

    Ok(())
}
