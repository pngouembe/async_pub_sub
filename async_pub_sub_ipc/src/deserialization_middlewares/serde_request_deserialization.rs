use std::fmt::Debug;
use std::sync::Arc;

use async_pub_sub::{Layer, Publisher, Request, Result, Subscriber};
use bytes::Bytes;
use futures::FutureExt;
use futures::future::BoxFuture;

pub trait BytesRequest: Request<Content = Bytes, Response = Bytes, SentResponse = Bytes>
where
    Self: Send,
    <Self as Request>::Response: Send,
    <Self as Request>::SentResponse: Send,
{
}

// Implement BytesRequest for NatsRequest
impl BytesRequest for crate::wire_subscribers::NatsRequest {}

pub struct SerdeRequestDeserializationLayer<Req, Rsp> {
    deserialization_function: Arc<dyn Fn(Bytes) -> Result<Req> + Send + Sync>,
    serialization_function: Arc<dyn Fn(&Rsp) -> Result<Bytes> + Send + Sync>,
}

impl<Req, Rsp> SerdeRequestDeserializationLayer<Req, Rsp> {
    pub fn new<DeserializationFunction, SerializationFunction, DErr, SErr, SOut>(
        deserialization_function: DeserializationFunction,
        serialization_function: SerializationFunction,
    ) -> Self
    where
        DeserializationFunction: for<'a> Fn(&'a [u8]) -> std::result::Result<Req, DErr> + Send + Sync + 'static,
        SerializationFunction: Fn(&Rsp) -> std::result::Result<SOut, SErr> + Send + Sync + 'static,
        DErr: std::error::Error + Send + Sync + 'static,
        SErr: std::error::Error + Send + Sync + 'static,
        Bytes: From<SOut>,
    {
        let deserialization_function = Arc::new(move |bytes: Bytes| {
            deserialization_function(&bytes).map_err(async_pub_sub::Error::from)
        });
        let serialization_function = Arc::new(move |value: &Rsp| {
            serialization_function(value)
                .map(Bytes::from)
                .map_err(async_pub_sub::Error::from)
        });
        Self {
            deserialization_function,
            serialization_function,
        }
    }

    pub fn serde_json() -> Self
    where
        Req: serde::de::DeserializeOwned,
        Rsp: serde::Serialize,
    {
        Self::new(
            |bytes: &[u8]| serde_json::from_slice(bytes),
            |value: &Rsp| serde_json::to_vec(value),
        )
    }
}

impl<Req, Rsp, S> Layer<S> for SerdeRequestDeserializationLayer<Req, Rsp> {
    type LayerType = SerdeRequestDeserializationSubscriber<Req, Rsp, S>;

    fn layer(self, subscriber: S) -> Self::LayerType {
        SerdeRequestDeserializationSubscriber {
            deserialization_function: self.deserialization_function,
            serialization_function: self.serialization_function,
            subscriber,
        }
    }
}

pub struct SerdeRequestDeserializationSubscriber<Req, Rsp, S> {
    deserialization_function: Arc<dyn Fn(Bytes) -> Result<Req> + Send + Sync>,
    serialization_function: Arc<dyn Fn(&Rsp) -> Result<Bytes> + Send + Sync>,
    subscriber: S,
}

impl<Req, Rsp, S> Subscriber for SerdeRequestDeserializationSubscriber<Req, Rsp, S>
where
    Req: Debug + Send + Sync + 'static,
    Rsp: Debug + Send + Sync + 'static,
    S: Subscriber + Send + Sync + 'static,
    S::Message: BytesRequest + Send + 'static,
{
    type Message = SerdeRequestWrapper<Req, Rsp, S::Message>;

    fn get_name(&self) -> &'static str {
        self.subscriber.get_name()
    }

    fn subscribe_to(
        &mut self,
        _publisher: &mut dyn Publisher<Message = Self::Message>,
    ) -> Result<()> {
        Err(
            "SerdeJsonDeserializationSubscriber cannot subscribe to publishers. It can only receive messages from the network.".into()
        )
    }

    fn receive(&mut self) -> BoxFuture<Self::Message> {
        let future_request = self.subscriber.receive();
        let deserialization_function = self.deserialization_function.clone();
        let serialization_function = self.serialization_function.clone();

        async move {
            let mut bytes_request = future_request.await;
            let serialized_content = bytes_request
                .take_content()
                .expect("Failed to get request content");
            let deserialized_content = (deserialization_function)(serialized_content)
                .expect("Failed to deserialize request content");

            SerdeRequestWrapper::new(deserialized_content, bytes_request, serialization_function)
        }
        .boxed()
    }
}

pub struct SerdeRequestWrapper<Req, Rsp, R> {
    content: Option<Req>,
    bytes_request: Option<R>,
    serialization_function: Arc<dyn Fn(&Rsp) -> Result<Bytes> + Send + Sync>,
}

impl<Req, Rsp, R> SerdeRequestWrapper<Req, Rsp, R>
where
    R: BytesRequest,
{
    pub fn new(
        content: Req,
        bytes_request: R,
        serialization_function: Arc<dyn Fn(&Rsp) -> Result<Bytes> + Send + Sync>,
    ) -> Self {
        Self {
            content: Some(content),
            bytes_request: Some(bytes_request),
            serialization_function,
        }
    }
}

impl<Req, Rsp, R> Request for SerdeRequestWrapper<Req, Rsp, R>
where
    Req: Send + 'static,
    Rsp: Send + 'static,
    R: BytesRequest + Send + 'static,
{
    type Content = Req;
    type Response = Rsp;
    type SentResponse = Rsp;

    fn take_response(self) -> (Self, BoxFuture<'static, Result<Self::Response>>) {
        panic!(
            "SerdeRequestWrapper does not support take_response - it's for server-side handling only"
        );
    }

    fn respond(
        mut self,
        response: Self::SentResponse,
    ) -> impl std::future::Future<Output = Result<()>> {
        async move {
            let bytes_request = self.bytes_request.take().expect("Request already used");
            let serialized_response = (self.serialization_function)(&response)?;
            bytes_request
                .respond(Bytes::from(serialized_response))
                .await
        }
    }

    fn take_content(&mut self) -> Option<Self::Content> {
        self.content.take()
    }
}
