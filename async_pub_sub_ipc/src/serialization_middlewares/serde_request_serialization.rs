use std::fmt::Debug;
use std::pin::Pin;

use async_pub_sub::{Layer, Publisher, Request, RequestImpl, Requester, Result};
use bytes::Bytes;
use futures::future::BoxFuture;
use futures::{FutureExt, Stream};

pub struct SerdeRequestSerializationLayer<Req, Rsp> {
    serialization_function: Box<dyn Fn(&Req) -> Result<Bytes> + Send + Sync>,
    deserialization_function: Box<dyn Fn(Bytes) -> Result<Rsp> + Send + Sync>,
}

impl<Req, Rsp> SerdeRequestSerializationLayer<Req, Rsp> {
    pub fn new<SF, DF, SErr, DErr, SOut>(
        serialization_function: SF,
        deserialization_function: DF,
    ) -> Self
    where
        SF: Fn(&Req) -> std::result::Result<SOut, SErr> + Send + Sync + 'static,
        DF: for<'a> Fn(&'a [u8]) -> std::result::Result<Rsp, DErr> + Send + Sync + 'static,
        SErr: std::error::Error + Send + Sync + 'static,
        DErr: std::error::Error + Send + Sync + 'static,
        Bytes: From<SOut>,
    {
        let serialization_function = Box::new(move |value: &Req| {
            serialization_function(value)
                .map(Bytes::from)
                .map_err(async_pub_sub::Error::from)
        });
        let deserialization_function = Box::new(move |bytes: Bytes| {
            deserialization_function(&bytes).map_err(async_pub_sub::Error::from)
        });
        Self {
            serialization_function,
            deserialization_function,
        }
    }

    pub fn serde_json() -> Self
    where
        Req: serde::Serialize,
        Rsp: serde::de::DeserializeOwned,
    {
        Self::new(
            |value: &Req| serde_json::to_vec(value),
            |bytes: &[u8]| serde_json::from_slice(bytes),
        )
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
    deserialization_function: Box<dyn Fn(Bytes) -> Result<Rsp> + Send + Sync>,
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

    fn publish(&self, mut message: Self::Message) -> BoxFuture<Result<()>> {
        async move {
            let content = message.take_content().unwrap();
            let serialized_content = (self.serialization_function)(&content)?;
            let (request, response) =
                RequestImpl::new(Bytes::from(serialized_content)).take_response();
            self.publisher.publish(request).await?;
            let response = response.await?;

            let deserialized_response = (self.deserialization_function)(response)?;

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
        mut request: Self::Message,
    ) -> impl Future<Output = Result<<Self::Message as Request>::Response>> {
        let content = request.take_content().unwrap();
        let serialized_content = (self.serialization_function)(&content);

        async move {
            let inner_request = RequestImpl::new(Bytes::from(serialized_content?));

            let response = self.publisher.request(inner_request).await?;

            let deserialized_response = (self.deserialization_function)(response)?;
            Ok(deserialized_response)
        }
    }
}
