use std::fmt::Debug;
use std::pin::Pin;

use async_pub_sub::{Layer, Publisher, Request, RequestImpl, Requester, Result};
use bytes::Bytes;
use futures::future::BoxFuture;
use futures::{FutureExt, Stream};

pub struct SerdeRequestSerializationLayer<Req, Rsp> {
    serialization_function: Box<dyn Fn(&Req) -> Result<Bytes> + Send + Sync>,
    deserialization_function: Box<dyn Fn(&Bytes) -> Result<Rsp> + Send + Sync>,
}

impl<Req, Rsp> SerdeRequestSerializationLayer<Req, Rsp> {
    pub fn new(
        serialization_function: impl Fn(&Req) -> Result<Bytes> + Send + Sync + 'static,
        deserialization_function: impl Fn(&Bytes) -> Result<Rsp> + Send + Sync + 'static,
    ) -> Self {
        let serialization_function = Box::new(serialization_function);
        let deserialization_function = Box::new(deserialization_function);
        Self {
            serialization_function,
            deserialization_function,
        }
    }
}

impl<Req, Rsp, P> Layer<P> for SerdeRequestSerializationLayer<Req, Rsp> {
    type LayerType = SerdeRequestSerializationPublisher<Req, Rsp, P>;

    fn layer(self, publisher: P) -> Self::LayerType {
        SerdeRequestSerializationPublisher {
            serialization_function: self.serialization_function,
            deserialization_function: self.deserialization_function,
            publisher,
        }
    }
}

pub struct SerdeRequestSerializationPublisher<Req, Rsp, P> {
    serialization_function: Box<dyn Fn(&Req) -> Result<Bytes> + Send + Sync>,
    deserialization_function: Box<dyn Fn(&Bytes) -> Result<Rsp> + Send + Sync>,
    publisher: P,
}

impl<Req, Rsp, P> Publisher for SerdeRequestSerializationPublisher<Req, Rsp, P>
where
    Req: Debug + Send + Sync + 'static,
    Rsp: Debug + Send + Sync + 'static,
    P: Publisher<Message = RequestImpl<Bytes, Bytes>> + Send + Sync + 'static,
{
    type Message = RequestImpl<Req, Rsp>;

    fn get_name(&self) -> &'static str {
        self.publisher.get_name()
    }

    fn publish(&self, message: Self::Message) -> BoxFuture<Result<()>> {
        async move {
            let content = message.get_content();
            let serialized_content = (self.serialization_function)(&content)?;
            let (request, response) = RequestImpl::new(serialized_content).take_response();
            self.publisher.publish(request).await?;
            let response = response.await?;

            let deserialized_response = (self.deserialization_function)(&response)?;

            message.respond(deserialized_response).await?;

            Ok(())
        }
        .boxed()
    }

    fn get_message_stream(
        &mut self,
        _subscriber_name: &'static str,
    ) -> Result<Pin<Box<dyn Stream<Item = Self::Message> + Send + Sync + 'static>>> {
        Err("SerdeJsonSerializationPublisher does not support message streams - use for publishing only".into())
    }
}

impl<Req, Rsp, P> Requester for SerdeRequestSerializationPublisher<Req, Rsp, P>
where
    Req: Debug + Send + Sync + 'static,
    Rsp: Debug + Send + Sync + 'static,
    P: Publisher<Message = RequestImpl<Bytes, Bytes>> + Requester + Send + Sync + 'static,
{
    fn request(
        &self,
        request: Self::Message,
    ) -> impl Future<Output = Result<<Self::Message as Request>::Response>> {
        let serialized_content = (self.serialization_function)(request.get_content());

        async move {
            let inner_request = RequestImpl::new(serialized_content?);

            let response = self.publisher.request(inner_request).await?;

            let deserialized_response = (self.deserialization_function)(&response)?;
            Ok(deserialized_response)
        }
    }
}
