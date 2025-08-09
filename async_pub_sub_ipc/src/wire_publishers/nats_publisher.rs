use crate::Result;
use async_pub_sub::Publisher;
use bytes::Bytes;
use futures::FutureExt;

pub struct NatsPublisher<T> {
    name: &'static str,
    nats_client: async_nats::Client,
    _marker: std::marker::PhantomData<T>,
}

impl<T> NatsPublisher<T> {
    pub async fn new(name: &'static str, nats_url: &str) -> Result<Self> {
        let nats_client = async_nats::connect(nats_url).await?;
        Ok(Self {
            name,
            nats_client,
            _marker: std::marker::PhantomData,
        })
    }
}

impl<T> Publisher for NatsPublisher<T>
where
    T: Sync,
{
    type Message = Bytes;

    fn get_name(&self) -> &'static str {
        self.name
    }

    fn publish(
        &self,
        message: Self::Message,
    ) -> futures::future::BoxFuture<async_pub_sub::Result<()>> {
        async move {
            let subject = std::any::type_name::<T>();
            self.nats_client.publish(subject, message).await?;
            Ok(())
        }
        .boxed()
    }

    fn get_message_stream(
        &mut self,
        _subscriber_name: &'static str,
    ) -> async_pub_sub::Result<
        std::pin::Pin<Box<dyn futures::Stream<Item = Self::Message> + Send + Sync + 'static>>,
    > {
        Err("Cannot subscribe to a Nats Publisher, it can only be used to publish over the network.".into())
    }
}
