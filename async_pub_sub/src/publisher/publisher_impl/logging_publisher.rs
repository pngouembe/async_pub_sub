use std::{fmt::Display, pin::Pin};

use futures::{future::BoxFuture, FutureExt, Stream};

use crate::{Publisher, Result, SimplePublisher};

pub struct LoggingPublisher<Message>
where
    Message: Display + Send + 'static,
{
    subscriber_name: Option<&'static str>,
    publisher: SimplePublisher<Message>,
}

impl<Message> LoggingPublisher<Message>
where
    Message: Display + Send + 'static,
{
    pub fn new(name: &'static str, buffer_size: usize) -> Self {
        Self {
            subscriber_name: None,
            publisher: SimplePublisher::new(name, buffer_size),
        }
    }
}

impl<Message> Publisher for LoggingPublisher<Message>
where
    Message: Display + Send + 'static,
{
    type Message = Message;

    fn get_name(&self) -> &'static str {
        self.publisher.get_name()
    }

    fn publish(&self, message: Message) -> BoxFuture<Result<()>> {
        async move {
            let message_str = format!("{}", &message);
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
        log::info!(
            "({}) <-> ({}): {}",
            self.publisher.get_name(),
            subscriber_name,
            std::any::type_name::<Message>()
        );
        Ok(stream)
    }
}
