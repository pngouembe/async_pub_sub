use std::fmt::Display;

use futures::{
    stream::{self, BoxStream, SelectAll},
    FutureExt, StreamExt,
};

use crate::{Publisher, Result, Subscriber};

// todo: create logging forwarder using a middleware pattern
pub struct LoggingForwarder<Message>
where
    Message: Display + Send + 'static,
{
    name: &'static str,
    messages: Option<SelectAll<BoxStream<'static, Message>>>,
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

impl<Message> Subscriber<Message> for LoggingForwarder<Message>
where
    Message: Display + Send + 'static,
{
    fn get_name(&self) -> &'static str {
        self.name
    }

    fn subscribe_to(&mut self, publisher: &mut impl Publisher<Message>) -> Result<()> {
        let stream = publisher.get_message_stream(self.name)?;

        // todo: Fix the unwrap
        self.messages.as_mut().unwrap().push(stream);

        Ok(())
    }

    fn receive(&mut self) -> impl std::future::Future<Output = Message> {
        async move { panic!("LoggingForwarder does not implement receive method") }
    }
}

impl<Message> Publisher<Message> for LoggingForwarder<Message>
where
    Message: Display + Send + 'static,
{
    fn get_name(&self) -> &'static str {
        self.name
    }

    fn publish_event(&self, _message: Message) -> futures::future::BoxFuture<Result<()>> {
        async move { panic!("LoggingForwarder does not implement publish method") }.boxed()
    }

    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<BoxStream<'static, Message>> {
        // todo: Fix the unwrap
        let messages = self.messages.take().unwrap();
        let name = self.name;

        let stream = stream::unfold(messages, move |mut messages| async move {
            let message = messages.select_next_some().await;
            log::info!("[{}] -> [{}]: {}", name, subscriber_name, message);
            Some((message, messages))
        })
        .boxed();

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
    use crate::{LoggingPublisher, LoggingSubscriber};

    use super::*;

    #[test_log::test(tokio::test)]
    async fn test_logging_forwarder() {
        let mut forwarder = LoggingForwarder::new("forwarder");
        let mut publisher = LoggingPublisher::new("publisher", 1);
        let mut subscriber = LoggingSubscriber::new("subscriber");

        forwarder.subscribe_to(&mut publisher).unwrap();
        subscriber.subscribe_to(&mut forwarder).unwrap();

        publisher.publish_event("Hello, World!").await.unwrap();

        let message = subscriber.receive().await;
        assert_eq!(message, "Hello, World!");
    }
}
