use std::{fmt::Display, pin::Pin};

use async_pub_sub::{Publisher, PublisherImpl, Request, Result, Subscriber, SubscriberImpl};
use futures::{
    stream::{self, SelectAll},
    FutureExt, Stream, StreamExt,
};

// TODO: fix the request response logging in the forwarder
struct LoggingForwarder<Message>
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

    fn subscribe_to(
        &mut self,
        publisher: &mut impl async_pub_sub::PublisherWrapper<Self::Message>,
    ) -> Result<()> {
        let stream = publisher.get_message_stream(self.name)?;

        self.messages.as_mut().unwrap().push(stream);

        Ok(())
    }

    async fn receive(&mut self) -> Message {
        panic!("LoggingForwarder does not implement receive method")
    }
}

impl<Message> Publisher for LoggingForwarder<Message>
where
    Message: Display + Send + 'static,
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

#[test_log::test(tokio::test)]
async fn test_message_forwarder() -> Result<()> {
    // -- Setup & Fixtures
    let mut subscriber = SubscriberImpl::<Request<i32, i32>>::new("subscriber");
    let mut forwarder = LoggingForwarder::new("forwarder");
    let mut publisher = PublisherImpl::new("publisher", 10);

    forwarder.subscribe_to(&mut publisher)?;
    subscriber.subscribe_to(&mut forwarder)?;

    // -- Exec
    let publisher_task = tokio::spawn(async move {
        let (request, response) = Request::new(42);
        publisher
            .publish(request)
            .await
            .expect("request published successfully");
        assert_eq!(response.await.expect("request successful"), 43);
    });

    let subscriber_task = tokio::spawn(async move {
        let request = subscriber.receive().await;
        let response = request.content + 1;

        request.respond(response);
    });

    // -- Check
    tokio::try_join!(publisher_task, subscriber_task)?;

    Ok(())
}
