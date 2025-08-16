use crate::{IpcRequestSubscriber, Result};
use async_nats::Message;
use async_pub_sub::{Publisher, Request, Subscriber};
use bytes::Bytes;
use futures::{FutureExt, StreamExt, future::BoxFuture};

pub struct NatsSubscriber<T> {
    name: &'static str,
    nats_client: async_nats::Client,
    nats_subscriber: async_nats::Subscriber,
    _marker: std::marker::PhantomData<T>,
}

impl<T> NatsSubscriber<T> {
    pub async fn new(name: &'static str, nats_url: &str) -> Result<Self> {
        let nats_client = async_nats::connect(nats_url).await?;
        let nats_subscriber = nats_client
            .subscribe(
                // TODO: create a sanitization function for subject names
                std::any::type_name::<T>()
                    .replace("<", "__")
                    .replace(">", "__")
                    .replace(" ", "__"),
            )
            .await?;
        Ok(Self {
            name,
            nats_client,
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

impl<T> IpcRequestSubscriber for NatsSubscriber<T>
where
    T: Send + Sync + 'static,
{
    fn get_name(&self) -> &'static str {
        self.name
    }

    fn receive_request(
        &mut self,
    ) -> BoxFuture<(
        Bytes,
        impl FnOnce(Bytes) -> BoxFuture<'static, ()> + Send + 'static,
    )> {
        let future_message = self.nats_subscriber.next();
        let nats_client = self.nats_client.clone();
        let name = self.name;
        async move {
            let msg = future_message.await.expect("Should receive a message");
            let content = msg.payload;
            log::debug!(
                "[{}] Received request from subject '{}': {:?}",
                name,
                msg.subject,
                content
            );
            (content, move |response| {
                async move {
                    if let Some(subject) = msg.reply {
                        log::debug!(
                            "[{}] Sending response to subject '{}': {:?}",
                            name,
                            subject,
                            response
                        );
                        nats_client
                            .publish(subject, response)
                            .await
                            .expect("Failed to publish response");
                    }
                    // Handle response logic here if needed
                }
                .boxed()
            })
        }
        .boxed()
    }
}

pub struct NatsRequestSubscriber<T> {
    name: &'static str,
    nats_client: async_nats::Client,
    nats_subscriber: async_nats::Subscriber,
    _marker: std::marker::PhantomData<T>,
}

impl<T> NatsRequestSubscriber<T> {
    pub async fn new(name: &'static str, nats_url: &str) -> Result<Self> {
        let nats_client = async_nats::connect(nats_url).await?;
        let nats_subscriber = nats_client
            .subscribe(
                std::any::type_name::<T>()
                    .replace("<", "__")
                    .replace(">", "__")
                    .replace(" ", "__"),
            )
            .await?;
        Ok(Self {
            name,
            nats_client,
            nats_subscriber,
            _marker: std::marker::PhantomData,
        })
    }
}

impl<T> Subscriber for NatsRequestSubscriber<T> {
    type Message = NatsRequest;

    fn get_name(&self) -> &'static str {
        self.name
    }

    fn subscribe_to(
        &mut self,
        _publisher: &mut dyn Publisher<Message = Self::Message>,
    ) -> async_pub_sub::Result<()> {
        Err("NatsSubscriber cannot subscribe to publishers, it can only be used to receive messages from the network.".into())
    }

    fn receive(&mut self) -> BoxFuture<Self::Message> {
        let future_message = self.nats_subscriber.next();
        let nats_client = self.nats_client.clone();
        async move {
            let msg = future_message.await.expect("Should receive a message");
            let request = NatsRequest::new(msg, nats_client).expect("Failed to create NatsRequest");
            request
        }
        .boxed()
    }
}

pub struct NatsRequest {
    content: Option<Bytes>,
    response: Box<dyn FnOnce(Bytes) -> BoxFuture<'static, ()> + Send>,
}

impl NatsRequest {
    pub fn new(message: Message, nats_client: async_nats::Client) -> Result<Self> {
        let reply_subject = message.reply.ok_or("Missing reply subject")?;
        let content = message.payload;
        let response = Box::new(move |response: Bytes| {
            async move {
                nats_client
                    .publish(reply_subject, response)
                    .await
                    .expect("Failed to publish response");
            }
            .boxed()
        });
        Ok(Self {
            content: Some(content),
            response,
        })
    }
}

impl Request for NatsRequest {
    type Content = Bytes;
    type Response = Bytes;
    type SentResponse = Bytes;

    fn take_response(self) -> (Self, BoxFuture<'static, Result<Self::Response>>) {
        panic!("NatsRequest does not support take_response");
    }

    fn respond(self, response: Self::SentResponse) -> impl Future<Output = Result<()>> {
        async move {
            (self.response)(response).await;
            Ok(())
        }
    }

    fn take_content(&mut self) -> Option<Self::Content> {
        self.content.take()
    }
}
