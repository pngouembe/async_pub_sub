use async_pub_sub::{Layer, Publisher};
use async_pub_sub::{Result, Subscriber};
use bytes::Bytes;
use futures::FutureExt;
use futures::future::BoxFuture;

pub struct SerdeJsonDeserializationLayer<Message> {
    _phantom: std::marker::PhantomData<Message>,
}

impl<Message> SerdeJsonDeserializationLayer<Message> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Message, S> Layer<S> for SerdeJsonDeserializationLayer<Message>
where
    S: Subscriber<Message = Bytes> + Send,
    Message: serde::de::DeserializeOwned + Send + Sync + 'static,
{
    type LayerType = SerdeJsonDeserializationSubscriber<Message, S>;

    fn layer(self, subscriber: S) -> Self::LayerType {
        SerdeJsonDeserializationSubscriber {
            subscriber,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct SerdeJsonDeserializationSubscriber<Message, S>
where
    S: Subscriber<Message = Bytes>,
{
    subscriber: S,
    _phantom: std::marker::PhantomData<Message>,
}

impl<Message, S> Subscriber for SerdeJsonDeserializationSubscriber<Message, S>
where
    S: Subscriber<Message = Bytes> + Sync,
    Message: serde::de::DeserializeOwned + Send + Sync + 'static,
{
    type Message = Message;

    fn get_name(&self) -> &'static str {
        self.subscriber.get_name()
    }

    fn subscribe_to(
        &mut self,
        _publisher: &mut dyn Publisher<Message = Self::Message>,
    ) -> Result<()> {
        Err(
            "SerdeJsonDeserializationSubscriber cannot subscribe to publishers. It can only receive messages from the network.".into()        )
    }

    fn receive(&mut self) -> BoxFuture<Self::Message> {
        let future_message = self.subscriber.receive();
        async move {
            let bytes = future_message.await;
            serde_json::from_slice(&bytes).expect("should deserialize JSON")
        }
        .boxed()
    }
}
