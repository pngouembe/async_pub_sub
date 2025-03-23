use std::{fmt::Display, pin::Pin};

use crate::{MultiPublisher, Publisher, Result, Subscriber};
use futures::{
    stream::{self, SelectAll},
    FutureExt, Stream, StreamExt,
};

// todo: create logging forwarder using a middleware pattern
pub struct LoggingForwarder<Message>
where
    Message: Display + Send + 'static,
{
    name: &'static str,
    messages: Option<SelectAll<Pin<Box<dyn Stream<Item = Message> + Send + Sync + 'static>>>>,
}

impl<Message> LoggingForwarder<Message>
where
    Message: Display + Send + 'static,
{
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            messages: Some(SelectAll::new()),
        }
    }
}

impl<Message> Subscriber for LoggingForwarder<Message>
where
    Message: Display + Send + 'static,
{
    type Message = Message;

    fn get_name(&self) -> &'static str {
        self.name
    }

    fn subscribe_to(&mut self, publisher: &mut impl MultiPublisher<Self::Message>) -> Result<()> {
        let stream = publisher.get_message_stream(self.name)?;

        // todo: Fix the unwrap
        self.messages.as_mut().unwrap().push(stream);

        Ok(())
    }

    async fn receive(&mut self) -> Message {
        panic!("LoggingForwarder does not implement receive method")
    }
}

impl<Message> Publisher for LoggingForwarder<Message>
where
    Message: Display + Send + Sync + 'static,
{
    type Message = Message;

    fn get_name(&self) -> &'static str {
        self.name
    }

    fn publish(&self, _message: Message) -> futures::future::BoxFuture<Result<()>> {
        async move { panic!("LoggingForwarder does not implement publish method") }.boxed()
    }

    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<Pin<Box<dyn Stream<Item = Message> + Send + Sync + 'static>>> {
        // todo: Fix the unwrap
        let messages = self.messages.take().unwrap();
        let name = self.name;

        let stream = Box::pin(stream::unfold(messages, move |mut messages| async move {
            let message = messages.select_next_some().await;
            log::info!("[{}] -> [{}]: {}", name, subscriber_name, message);
            Some((message, messages))
        }));

        log::info!(
            "({}) <-> ({}): {}",
            self.name,
            subscriber_name,
            std::any::type_name::<Message>()
        );

        Ok(stream)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        utils::forwarder::LoggingForwarder, LoggingPublisher, LoggingSubscriber, Publisher,
        Subscriber,
    };

    #[test_log::test(tokio::test)]
    async fn test_logging_forwarder() {
        let mut forwarder = LoggingForwarder::new("forwarder");
        let mut publisher = LoggingPublisher::new("publisher", 1);
        let mut subscriber = LoggingSubscriber::new("subscriber");

        forwarder.subscribe_to(&mut publisher).unwrap();
        subscriber.subscribe_to(&mut forwarder).unwrap();

        publisher.publish("Hello, World!").await.unwrap();

        let message = subscriber.receive().await;
        assert_eq!(message, "Hello, World!");
    }
}
