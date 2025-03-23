use std::fmt::Display;

use crate::{MultiPublisher, Result, Subscriber};

use super::SimpleSubscriber;

pub struct LoggingSubscriber<Message>
where
    Message: Send + 'static,
{
    publisher_name: Option<&'static str>,
    subscriber: SimpleSubscriber<Message>,
}

impl<Message> LoggingSubscriber<Message>
where
    Message: Send + 'static,
{
    pub fn new(name: &'static str) -> Self {
        let subscriber = SimpleSubscriber::new(name);

        Self {
            publisher_name: None,
            subscriber,
        }
    }
}

impl<Message> Subscriber for LoggingSubscriber<Message>
where
    Message: Display + Send + 'static,
{
    type Message = Message;

    fn get_name(&self) -> &'static str {
        self.subscriber.get_name()
    }

    fn subscribe_to(&mut self, publisher: &mut impl MultiPublisher<Self::Message>) -> Result<()> {
        self.subscriber.subscribe_to(publisher)?;
        self.publisher_name = Some(publisher.get_name());
        log::info!(
            "({}) <-> ({}): {}",
            self.subscriber.get_name(),
            publisher.get_name(),
            std::any::type_name::<Message>()
        );
        Ok(())
    }

    fn receive(&mut self) -> impl std::future::Future<Output = Message> {
        let publisher_name = self.publisher_name.expect("publisher name should be known");
        async move {
            let message = self.subscriber.receive().await;
            log::info!(
                "[{}] <- [{}]: {}",
                self.subscriber.get_name(),
                publisher_name,
                message
            );
            message
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::SimplePublisher;

    use super::*;

    #[test_log::test(tokio::test)]
    async fn test_logging_subscriber() -> Result<()> {
        // -- Setup & Fixtures
        let mut subscriber = LoggingSubscriber::new("subscriber");
        let mut publisher = SimplePublisher::new("publisher", 10);

        subscriber.subscribe_to(&mut publisher)?;

        publisher.publish(42).await?;

        let message = subscriber.receive().await;

        assert_eq!(message, 42);

        Ok(())
    }
}
