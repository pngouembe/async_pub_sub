use crate::Result;
use async_pub_sub::{Publisher, Subscriber};
use bytes::Bytes;
use futures::{FutureExt, StreamExt};

pub struct NatsSubscriber<T> {
    name: &'static str,
    _nats_client: async_nats::Client,
    nats_subscriber: async_nats::Subscriber,
    _marker: std::marker::PhantomData<T>,
}

impl<T> NatsSubscriber<T> {
    pub async fn new(name: &'static str, nats_url: &str) -> Result<Self> {
        let _nats_client = async_nats::connect(nats_url).await?;
        let nats_subscriber = _nats_client.subscribe(std::any::type_name::<T>()).await?;
        Ok(Self {
            name,
            _nats_client,
            nats_subscriber,
            _marker: std::marker::PhantomData,
        })
    }
}

impl<T> Subscriber for NatsSubscriber<T>
where
    T: Send + Sync,
{
    type Message = Bytes;

    fn get_name(&self) -> &'static str {
        self.name
    }

    fn subscribe_to(
        &mut self,
        _publisher: &mut dyn Publisher<Message = Self::Message>,
    ) -> async_pub_sub::Result<()> {
        Err("NatsSubscriber cannot subscribe to publishers, it can only be used to receive messages from the network.".into())
    }

    fn receive(&mut self) -> futures::future::BoxFuture<Self::Message> {
        async move {
            let msg = self
                .nats_subscriber
                .next()
                .await
                .expect("Should receive a message");
            msg.payload
        }
        .boxed()
    }
}
