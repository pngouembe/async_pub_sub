use std::fmt::Debug;

use async_pub_sub::{Layer, Publisher, Request};
use async_pub_sub::{Result, Subscriber};
use futures::future::BoxFuture;

use crate::IpcRequestSubscriber;

pub struct SerdeJsonRequestDeserializationLayer<Req, Rsp> {
    _phantom: std::marker::PhantomData<(Req, Rsp)>,
}

impl<Req, Rsp> SerdeJsonRequestDeserializationLayer<Req, Rsp> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Req, Rsp, S> Layer<S> for SerdeJsonRequestDeserializationLayer<Req, Rsp>
where
    S: IpcRequestSubscriber + Send,
    Req: serde::de::DeserializeOwned + Send + Sync + 'static,
    Rsp: serde::ser::Serialize + Send + Sync + 'static,
{
    type LayerType = SerdeJsonRequestDeserializationSubscriber<Req, Rsp, S>;

    fn layer(&self, subscriber: S) -> Self::LayerType {
        SerdeJsonRequestDeserializationSubscriber {
            subscriber,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct SerdeJsonRequestDeserializationSubscriber<Req, Rsp, S>
where
    S: IpcRequestSubscriber,
{
    subscriber: S,
    _phantom: std::marker::PhantomData<(Req, Rsp)>,
}

impl<Req, Rsp, S> Subscriber for SerdeJsonRequestDeserializationSubscriber<Req, Rsp, S>
where
    S: IpcRequestSubscriber + Sync,
    Req: Debug + serde::de::DeserializeOwned + Send + Sync + 'static,
    Rsp: Debug + serde::ser::Serialize + Send + Sync + 'static,
{
    type Message = CustomRequest<Req, Rsp>;

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
        let future_request = self.subscriber.receive_request();

        Box::pin(async move {
            let (bytes, response_callback) = future_request.await;
            let content = serde_json::from_slice(&bytes).expect("should deserialize JSON");

            let request = CustomRequest {
                content,
                response_callback: Box::new(move |response: Rsp| {
                    Box::pin(async move {
                        let serialized_response =
                            serde_json::to_string(&response).expect("should serialize JSON");
                        response_callback(serialized_response.into()).await
                    })
                }),
            };

            request
        })
    }
}

pub struct CustomRequest<Req, Rsp> {
    pub content: Req,
    response_callback: Box<dyn FnOnce(Rsp) -> futures::future::BoxFuture<'static, ()> + Send>,
}

impl<Req, Rsp> Request for CustomRequest<Req, Rsp>
where
    Rsp: Send,
{
    type Response = Rsp;

    fn respond(self, response: Self::Response) -> impl futures::Future<Output = Result<()>> {
        async move {
            (self.response_callback)(response).await;
            Ok(())
        }
    }
}
