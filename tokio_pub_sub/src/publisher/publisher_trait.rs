use std::pin::Pin;

use futures::{future::BoxFuture, Stream};

use crate::Result;

pub trait Publisher {
    type Message: Send + 'static;

    fn get_name(&self) -> &'static str;

    fn publish_event(&self, message: Self::Message) -> BoxFuture<Result<()>>;

    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<Pin<Box<dyn Stream<Item = Self::Message> + Send + Sync + 'static>>>;
}

impl<T> Publisher for T
where
    T: std::ops::Deref + std::ops::DerefMut,
    T::Target: Publisher,
{
    type Message = <T::Target as Publisher>::Message;

    fn get_name(&self) -> &'static str {
        (**self).get_name()
    }

    fn publish_event(&self, message: Self::Message) -> BoxFuture<Result<()>> {
        (**self).publish_event(message)
    }

    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<Pin<Box<dyn Stream<Item = Self::Message> + Send + Sync + 'static>>> {
        (**self).get_message_stream(subscriber_name)
    }
}

pub trait PublisherLayer<InnerPublisherType>
where
    InnerPublisherType: Publisher,
{
    type PublisherType: Publisher;
    fn layer(&self, publisher: InnerPublisherType) -> Self::PublisherType;
}

pub trait MultiPublisher<Message>
where
    Message: Send + 'static,
{
    fn get_publisher(&self) -> &impl Publisher<Message = Message>;

    fn get_publisher_mut(&mut self) -> &mut impl Publisher<Message = Message>;

    fn get_name(&self) -> &'static str {
        Publisher::get_name(self.get_publisher())
    }

    fn publish_event(&self, message: Message) -> futures::future::BoxFuture<Result<()>> {
        Publisher::publish_event(self.get_publisher(), message)
    }

    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<std::pin::Pin<Box<dyn futures::Stream<Item = Message> + Send + Sync + 'static>>>
    {
        Publisher::get_message_stream(self.get_publisher_mut(), subscriber_name)
    }
}

// TODO: Rename
impl<T> MultiPublisher<T::Message> for T
where
    T: Publisher,
{
    fn get_publisher(&self) -> &impl Publisher<Message = T::Message> {
        self
    }

    fn get_publisher_mut(&mut self) -> &mut impl Publisher<Message = T::Message> {
        self
    }
}
