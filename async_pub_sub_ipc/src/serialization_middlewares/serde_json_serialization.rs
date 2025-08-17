use async_pub_sub::{Layer, MappedPublisher, PublisherMapLayer};
use bytes::Bytes;

pub struct SerdeJsonSerializationLayer<Message> {
    inner: PublisherMapLayer<Message, Bytes>,
}

pub type SerdeJsonSerializationPublisher<Message, P> = MappedPublisher<Message, Bytes, P>;

impl<Message> Default for SerdeJsonSerializationLayer<Message>
where
    Message: serde::Serialize + Send + Sync + 'static,
 {
    fn default() -> Self {
        Self::new()
    }
}

impl<Message> SerdeJsonSerializationLayer<Message>
where
    Message: serde::Serialize + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            inner: PublisherMapLayer::new(|message: Message| {
                let serialized = serde_json::to_string(&message)?;
                Ok(Bytes::from(serialized))
            }),
        }
    }
}

impl<Message, P> Layer<P> for SerdeJsonSerializationLayer<Message>
where
    Message: serde::Serialize + Send + Sync + 'static,
    P: async_pub_sub::Publisher<InputMessage = Bytes> + Send,
{
    type LayerType = SerdeJsonSerializationPublisher<Message, P>;

    fn layer(self, publisher: P) -> Self::LayerType {
        self.inner.layer(publisher)
    }
}
