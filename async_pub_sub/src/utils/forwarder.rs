use std::{fmt::Display, pin::Pin};

use crate::{MultiPublisher, Publisher, Result, Subscriber, SubscriberImpl};
use futures::{stream, FutureExt, Stream};

// TODO: create logging forwarder using a middleware pattern
// TODO: Add the possibility to publisher from the forwarder using a middleware pattern
pub struct LoggingForwarder<Message>
where
    Message: Display + Send + 'static,
{
    name: &'static str,
    subscriber_name: Option<&'static str>,
    subscriber: Option<SubscriberImpl<Message>>,
}

impl<Message> LoggingForwarder<Message>
where
    Message: Display + Send + 'static,
{
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            subscriber_name: None,
            subscriber: Some(SubscriberImpl::new(name)),
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
        let Some(subscriber) = self.subscriber.as_mut() else {
            let subscriber_name = self
                .subscriber_name
                .expect("the subscriber name should be known at this point");

            return Err(format!(
                "{} forwarder has already been bound to {}, subscribe to {} before {} subscribes to it",
                self.name,
                subscriber_name,
                self.name,
                subscriber_name
            )
            .into());
        };

        subscriber.subscribe_to(publisher)
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
        let Some(subscriber) = self.subscriber.take() else {
            return Err(format!(
                "{} forwarder can only be bound to one subscriber (already bound to {})",
                self.name,
                self.subscriber_name
                    .expect("the subscriber name should be known at this point")
            )
            .into());
        };
        self.subscriber_name = Some(subscriber_name);

        let name = self.name;
        let stream = Box::pin(stream::unfold(
            subscriber,
            move |mut subscriber| async move {
                let message = subscriber.receive().await;
                log::info!("[{}] -> [{}]: {}", name, subscriber_name, message);
                Some((message, subscriber))
            },
        ));

        log::info!(
            "({}) <-> ({}): {}",
            self.name,
            subscriber_name,
            std::any::type_name::<Message>()
        );

        Ok(stream)
    }
}
