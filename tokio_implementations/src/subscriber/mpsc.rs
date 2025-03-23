use std::{future::Future, pin::Pin};

use futures::{stream::SelectAll, Stream, StreamExt};

use async_pub_sub::{MultiPublisher, Result, Subscriber};

// TODO: Bring this back to the async_pub_sub crate as it is a generic implementation

pub struct MpscSubscriber<Message>
where
    Message: Send + 'static,
{
    name: &'static str,
    messages: SelectAll<Pin<Box<dyn Stream<Item = Message> + Send + Sync + 'static>>>,
}

impl<Message> MpscSubscriber<Message>
where
    Message: Send + 'static,
{
    pub fn new(name: &'static str) -> Self {
        let messages = SelectAll::new();
        Self { name, messages }
    }

    pub fn subscribe_to(&mut self, publisher: &mut impl MultiPublisher<Message>) -> Result<()> {
        let stream = publisher.get_message_stream(self.name)?;
        self.messages.push(stream);
        Ok(())
    }

    pub async fn receive(&mut self) -> Message {
        self.messages.select_next_some().await
    }
}

impl<Message> Subscriber for MpscSubscriber<Message>
where
    Message: Send + 'static,
{
    type Message = Message;

    fn get_name(&self) -> &'static str {
        self.name
    }

    fn subscribe_to(&mut self, publisher: &mut impl MultiPublisher<Self::Message>) -> Result<()> {
        MpscSubscriber::subscribe_to(self, publisher)
    }

    fn receive(&mut self) -> impl Future<Output = Message> {
        MpscSubscriber::receive(self)
    }
}
