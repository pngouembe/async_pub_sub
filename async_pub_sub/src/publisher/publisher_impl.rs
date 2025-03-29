use futures::{FutureExt, SinkExt};

use super::Publisher;
use crate::Result;

pub struct PublisherImpl<Message>
where
    Message: Send + 'static,
{
    name: &'static str,
    subscriber_name: Option<&'static str>,
    sender: futures::channel::mpsc::Sender<Message>,
    receiver: Option<futures::channel::mpsc::Receiver<Message>>,
}

impl<Message> PublisherImpl<Message>
where
    Message: Send + 'static,
{
    pub fn new(name: &'static str, buffer_size: usize) -> Self {
        let (sender, receiver) = futures::channel::mpsc::channel(buffer_size);
        Self {
            name,
            subscriber_name: None,
            sender,
            receiver: Some(receiver),
        }
    }

    pub async fn publish(&self, message: Message) -> Result<()> {
        self.sender.clone().send(message).await?;
        Ok(())
    }
}

impl<Message> Publisher for PublisherImpl<Message>
where
    Message: Send + Sync + 'static,
{
    type Message = Message;

    fn get_name(&self) -> &'static str {
        self.name
    }

    fn publish(&self, message: Self::Message) -> futures::future::BoxFuture<Result<()>> {
        let mut sender = self.sender.clone();
        async move {
            sender
                .send(message)
                .await
                .map_err(|err| format!("Failed to send message (err: {})", err).into())
        }
        .boxed()
    }

    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<std::pin::Pin<Box<dyn futures::Stream<Item = Self::Message> + Send + Sync + 'static>>>
    {
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

        Ok(Box::pin(receiver))
    }
}
