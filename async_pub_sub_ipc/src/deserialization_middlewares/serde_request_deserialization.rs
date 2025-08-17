use std::fmt::Debug;

use async_pub_sub::{
    Layer, Request, Subscriber, SubscriberRequestMapLayer, SubscriberRequestMapSubscriber,
};
use bytes::Bytes;

pub struct SerdeRequestDeserializationLayer<Req, Rsp> {
    inner: SubscriberRequestMapLayer<Bytes, Bytes, Req, Rsp>,
}

pub type SerdeRequestDeserializationSubscriber<Req, Rsp, S> =
    SubscriberRequestMapSubscriber<Req, Rsp, S>;

impl<Req, Rsp> SerdeRequestDeserializationLayer<Req, Rsp>
where
    Req: Debug + Send + Sync + 'static,
    Rsp: Debug + Send + Sync + 'static,
{
    pub fn new<DeserializationFunction, SerializationFunction, DErr, SErr, SOut>(
        deserialization_function: DeserializationFunction,
        serialization_function: SerializationFunction,
    ) -> Self
    where
        DeserializationFunction:
            for<'a> Fn(&'a [u8]) -> std::result::Result<Req, DErr> + Send + Sync + 'static,
        SerializationFunction: Fn(&Rsp) -> std::result::Result<SOut, SErr> + Send + Sync + 'static,
        DErr: std::error::Error + Send + Sync + 'static,
        SErr: std::error::Error + Send + Sync + 'static,
        Bytes: From<SOut>,
    {
        let request_map = move |bytes: Bytes| {
            deserialization_function(&bytes).map_err(async_pub_sub::Error::from)
        };
        let response_map = move |value: Rsp| {
            serialization_function(&value)
                .map(Bytes::from)
                .map_err(async_pub_sub::Error::from)
        };

        Self {
            inner: SubscriberRequestMapLayer::new(request_map, response_map),
        }
    }

    pub fn serde_json() -> Self
    where
        Req: serde::de::DeserializeOwned + Debug + Send + Sync + 'static,
        Rsp: serde::Serialize + Debug + Send + Sync + 'static,
    {
        let request_map =
            |bytes: Bytes| serde_json::from_slice(&bytes).map_err(async_pub_sub::Error::from);
        let response_map = |value: Rsp| {
            serde_json::to_vec(&value)
                .map(Bytes::from)
                .map_err(async_pub_sub::Error::from)
        };

        Self {
            inner: SubscriberRequestMapLayer::new(request_map, response_map),
        }
    }
}

impl<Req, Rsp, S> Layer<S> for SerdeRequestDeserializationLayer<Req, Rsp>
where
    Req: Debug + Send + Sync + 'static,
    Rsp: Debug + Send + Sync + 'static,
    S: Subscriber + Send,
    S::OutputMessage:
        Request<Content = Bytes, Response = Bytes, SentResponse = Bytes> + Send + 'static,
{
    type LayerType = SerdeRequestDeserializationSubscriber<Req, Rsp, S>;

    fn layer(self, subscriber: S) -> Self::LayerType {
        self.inner.layer(subscriber)
    }
}
