use crate::{IpcRequestPublisher, Result};
use async_pub_sub::{Publisher, Request, RequestImpl, Requester};
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
            let subject = std::any::type_name::<T>()
                .replace("<", "__")
                .replace(">", "__")
                .replace(" ", "__");
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

impl<T> Requester for NatsPublisher<T>
where
    Self: Publisher,
    <Self as Publisher>::Message: Request<Content = Bytes, Response = Bytes>,
{
    fn request(
        &self,
        request: Self::Message,
    ) -> impl Future<Output = async_pub_sub::Result<<Self::Message as async_pub_sub::Request>::Response>>
    {
        async move {
            let content = request.get_content().clone();
            let subject = std::any::type_name::<T>()
                .replace("<", "__")
                .replace(">", "__")
                .replace(" ", "__");
            let response = self
                .nats_client
                .request(subject, content)
                .await
                .expect("failed to send request");

            Ok(response.payload)
        }
    }
}

impl<T> IpcRequestPublisher for NatsPublisher<T>
where
    T: Send + Sync + 'static,
{
    fn get_name(&self) -> &'static str {
        self.name
    }

    fn publish_request(
        &self,
        request: Bytes,
        response_callback: impl FnOnce(Bytes) -> std::result::Result<(), String> + Send + 'static,
    ) -> impl Future<Output = Result<()>> + Send {
        let subject = std::any::type_name::<T>()
            .replace("<", "__")
            .replace(">", "__")
            .replace(" ", "__");

        log::debug!(
            "[{}] Publishing request to subject '{}': {:?}",
            self.name,
            subject,
            request
        );

        async move {
            let response = self.nats_client.request(subject, request).await?;
            response_callback(response.payload)?;
            Ok(())
        }
        .boxed()
    }
}

pub struct NatsRequestPublisher<T> {
    name: &'static str,
    nats_client: async_nats::Client,
    _marker: std::marker::PhantomData<T>,
}

impl<T> NatsRequestPublisher<T> {
    pub async fn new(name: &'static str, nats_url: &str) -> Result<Self> {
        let nats_client = async_nats::connect(nats_url).await?;
        Ok(Self {
            name,
            nats_client,
            _marker: std::marker::PhantomData,
        })
    }
}

impl<T> Publisher for NatsRequestPublisher<T> {
    type Message = RequestImpl<Bytes, Bytes>;

    fn get_name(&self) -> &'static str {
        self.name
    }

    fn publish(
        &self,
        message: Self::Message,
    ) -> futures::future::BoxFuture<async_pub_sub::Result<()>> {
        let nats_client = self.nats_client.clone();
        async move {
            let subject = std::any::type_name::<T>()
                .replace("<", "__")
                .replace(">", "__")
                .replace(" ", "__");
            let content = message.get_content().clone();
            let response = nats_client.request(subject, content).await?;
            message.respond(response.payload).await?;

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

impl<T> Requester for NatsRequestPublisher<T>
where
    Self: Publisher,
    <Self as Publisher>::Message: Request<Content = Bytes, Response = Bytes>,
{
    fn request(
        &self,
        request: Self::Message,
    ) -> impl Future<Output = async_pub_sub::Result<<Self::Message as async_pub_sub::Request>::Response>>
    {
        let nats_client = self.nats_client.clone();
        async move {
            let content = request.get_content().clone();
            let subject = std::any::type_name::<T>()
                .replace("<", "__")
                .replace(">", "__")
                .replace(" ", "__");
            let response = nats_client.request(subject, content).await?;

            Ok(response.payload)
        }
    }
}
