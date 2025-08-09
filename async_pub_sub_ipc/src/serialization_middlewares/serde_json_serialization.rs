use std::pin::Pin;

use async_pub_sub::Result;
use async_pub_sub::{Layer, Publisher};
use bytes::Bytes;
use futures::future::BoxFuture;
use futures::{FutureExt, Stream};

pub struct SerdeJsonSerializationLayer<Message> {
    _phantom: std::marker::PhantomData<Message>,
}

impl<Message> SerdeJsonSerializationLayer<Message> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Message, P> Layer<P> for SerdeJsonSerializationLayer<Message>
where
    P: Publisher<Message = Bytes> + Send,
    Message: serde::Serialize + Send + Sync + 'static,
{
    type LayerType = SerdeJsonSerializationPublisher<Message, P>;

    fn layer(&self, publisher: P) -> Self::LayerType {
        SerdeJsonSerializationPublisher {
            publisher,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct SerdeJsonSerializationPublisher<Message, P>
where
    P: Publisher<Message = Bytes>,
{
    publisher: P,
    _phantom: std::marker::PhantomData<Message>,
}

impl<Message, P> Publisher for SerdeJsonSerializationPublisher<Message, P>
where
    P: Publisher<Message = Bytes> + Sync,
    Message: serde::Serialize + Send + Sync + 'static,
{
    type Message = Message;

    fn get_name(&self) -> &'static str {
        self.publisher.get_name()
    }

    fn publish(&self, message: Self::Message) -> BoxFuture<Result<()>> {
        async move {
            let serialized_message = serde_json::to_string(&message)?;
            self.publisher.publish(serialized_message.into()).await
        }
        .boxed()
    }

    fn get_message_stream(
        &mut self,
        _subscriber_name: &'static str,
    ) -> Result<Pin<Box<dyn Stream<Item = Self::Message> + Send + Sync + 'static>>> {
        // This serialization publisher cannot provide a deserialized message stream
        // since it only handles serialization. A corresponding deserialization
        // subscriber middleware would be needed for the receive side.
        Err("SerdeJsonSerializationPublisher does not support message streams - use for publishing only".into())
    }
}
