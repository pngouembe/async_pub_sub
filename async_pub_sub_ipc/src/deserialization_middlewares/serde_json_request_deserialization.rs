use std::fmt::Debug;

use crate::deserialization_middlewares::serde_request_deserialization::{
    SerdeRequestDeserializationLayer, SerdeRequestDeserializationSubscriber,
};

pub struct SerdeJsonRequestDeserializationLayer<Req, Rsp> {
    inner: SerdeRequestDeserializationLayer<Req, Rsp>,
}

pub type SerdeJsonRequestDeserializationSubscriber<Req, Rsp, S> =
    SerdeRequestDeserializationSubscriber<Req, Rsp, S>;

impl<Req, Rsp> Default for SerdeJsonRequestDeserializationLayer<Req, Rsp>
where
    Req: serde::de::DeserializeOwned + Debug + Send + Sync + 'static,
    Rsp: serde::Serialize + Debug + Send + Sync + 'static,
 {
    fn default() -> Self {
        Self::new()
    }
}

impl<Req, Rsp> SerdeJsonRequestDeserializationLayer<Req, Rsp>
where
    Req: serde::de::DeserializeOwned + Debug + Send + Sync + 'static,
    Rsp: serde::Serialize + Debug + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            inner: SerdeRequestDeserializationLayer::serde_json(),
        }
    }
}

impl<Req, Rsp, S> async_pub_sub::Layer<S> for SerdeJsonRequestDeserializationLayer<Req, Rsp>
where
    Req: serde::de::DeserializeOwned + Debug + Send + Sync + 'static,
    Rsp: serde::Serialize + Debug + Send + Sync + 'static,
    S: async_pub_sub::Subscriber + Send,
    S::OutputMessage: async_pub_sub::Request<
            Content = bytes::Bytes,
            Response = bytes::Bytes,
            SentResponse = bytes::Bytes,
        > + Send
        + 'static,
{
    type LayerType = SerdeJsonRequestDeserializationSubscriber<Req, Rsp, S>;

    fn layer(self, subscriber: S) -> Self::LayerType {
        self.inner.layer(subscriber)
    }
}
