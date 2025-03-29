use std::future::Future;

use crate::{PublisherWrapper, Result};

pub trait Subscriber {
    type Message: Send + 'static;

    fn get_name(&self) -> &'static str;

    fn subscribe_to(&mut self, publisher: &mut impl PublisherWrapper<Self::Message>) -> Result<()>;

    fn receive(&mut self) -> impl Future<Output = Self::Message> + Send;
}

pub trait MultiSubscriber<Message>
where
    Message: Send + 'static,
{
    fn get_subscriber(&self) -> &impl Subscriber<Message = Message>;

    fn get_subscriber_mut(&mut self) -> &mut impl Subscriber<Message = Message>;

    fn get_name(&self) -> &'static str {
        Subscriber::get_name(self.get_subscriber())
    }

    fn subscribe_to(&mut self, publisher: &mut impl PublisherWrapper<Message>) -> Result<()> {
        Subscriber::subscribe_to(self.get_subscriber_mut(), publisher)
    }

    fn receive(&mut self) -> impl Future<Output = Message> {
        Subscriber::receive(self.get_subscriber_mut())
    }
}

// TODO: Rename
impl<T> MultiSubscriber<T::Message> for T
where
    T: Subscriber,
{
    fn get_subscriber(&self) -> &impl Subscriber<Message = T::Message> {
        self
    }

    fn get_subscriber_mut(&mut self) -> &mut impl Subscriber<Message = T::Message> {
        self
    }
}
