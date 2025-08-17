use async_pub_sub::{Layer, Subscriber, SubscriberMapLayer, SubscriberMapSubscriber};
use bytes::Bytes;

pub struct SerdeJsonDeserializationLayer<Message> {
    inner: SubscriberMapLayer<Bytes, Message>,
}

pub type SerdeJsonDeserializationSubscriber<Message, S> = SubscriberMapSubscriber<Message, S>;

impl<Message> Default for SerdeJsonDeserializationLayer<Message>
where
    Message: serde::de::DeserializeOwned + Send + Sync + 'static,
 {
    fn default() -> Self {
        Self::new()
    }
}

impl<Message> SerdeJsonDeserializationLayer<Message>
where
    Message: serde::de::DeserializeOwned + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            inner: SubscriberMapLayer::new(|bytes: Bytes| {
                let message: Message = serde_json::from_slice(&bytes)?;
                Ok(message)
            }),
        }
    }
}

impl<Message, S> Layer<S> for SerdeJsonDeserializationLayer<Message>
where
    Message: serde::de::DeserializeOwned + Send + Sync + 'static,
    S: Subscriber<OutputMessage = Bytes> + Send,
{
    type LayerType = SerdeJsonDeserializationSubscriber<Message, S>;

    fn layer(self, subscriber: S) -> Self::LayerType {
        self.inner.layer(subscriber)
    }
}
