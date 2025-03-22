use std::pin::Pin;

use futures::{
    future::BoxFuture,
    stream::{self},
    FutureExt, Stream,
};

use crate::{Publisher, Result};

pub struct SimplePublisher<Message>
where
    Message: Send + 'static,
{
    name: &'static str,
    subscriber_name: Option<&'static str>,
    sender: tokio::sync::mpsc::Sender<Message>,
    receiver: Option<tokio::sync::mpsc::Receiver<Message>>,
}

impl<Message> SimplePublisher<Message>
where
    Message: Send + 'static,
{
    pub fn new(name: &'static str, buffer_size: usize) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(buffer_size);
        Self {
            name,
            subscriber_name: None,
            sender,
            receiver: Some(receiver),
        }
    }

    pub async fn publish(&self, message: Message) -> Result<()> {
        self.sender.send(message).await?;
        Ok(())
    }

    pub fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<Pin<Box<dyn Stream<Item = Message> + Send + Sync + 'static>>> {
        let Some(receiver) = self.receiver.take() else {
            return Err(format!(
                "{} publisher can only be bound to one subscriber (already bound to {})",
                self.name,
                self.subscriber_name
                    .expect("the subscriber name should be known at this point")
            )
            .into());
        };

        self.subscriber_name = Some(subscriber_name);

        Ok(Box::pin(stream::unfold(
            receiver,
            |mut receiver| async move { receiver.recv().await.map(|message| (message, receiver)) },
        )))
    }

    pub fn get_name(&self) -> &'static str {
        self.name
    }
}

impl<Message> Publisher for SimplePublisher<Message>
where
    Message: Send,
{
    type Message = Message;

    fn get_name(&self) -> &'static str {
        self.name
    }

    fn publish(&self, message: Message) -> BoxFuture<Result<()>> {
        SimplePublisher::publish(self, message).boxed()
    }

    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<Pin<Box<dyn Stream<Item = Message> + Send + Sync + 'static>>> {
        SimplePublisher::get_message_stream(self, subscriber_name)
    }
}
