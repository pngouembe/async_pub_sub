use std::{fmt::Display, pin::Pin};

use futures::{future::BoxFuture, FutureExt, Stream};

use crate::Result;

use super::publisher_types::Request;

pub trait Publisher {
    type Message: Send + 'static;

    fn get_name(&self) -> &'static str;

    fn publish_event(&self, message: Self::Message) -> BoxFuture<Result<()>>;

    fn publish_request<Req, Rsp>(&self, request: Req) -> BoxFuture<Result<Rsp>>
    where
        Req: Display + Send + 'static,
        Rsp: Display + Send + 'static,
        Self::Message: From<Request<Req, Rsp>>,
        Self: Sync,
    {
        async move {
            let (request, response) = Request::<Req, Rsp>::new(request);

            self.publish_event(request.into()).await?;

            let response = response.await?;
            Ok(response)
        }
        .boxed()
    }

    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<Pin<Box<dyn Stream<Item = Self::Message> + Send + Sync + 'static>>>;
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
