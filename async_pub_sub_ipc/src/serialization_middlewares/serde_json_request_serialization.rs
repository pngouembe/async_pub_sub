use std::fmt::Debug;
use std::pin::Pin;

use async_pub_sub::{Layer, Publisher, RequestImpl, Result};
use futures::future::BoxFuture;
use futures::{FutureExt, Stream};

use crate::IpcRequestPublisher;

pub struct SerdeJsonRequestSerializationLayer<Req, Rsp> {
    _phantom: std::marker::PhantomData<(Req, Rsp)>,
}

impl<Req, Rsp> SerdeJsonRequestSerializationLayer<Req, Rsp> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Req, Rsp, P> Layer<P> for SerdeJsonRequestSerializationLayer<Req, Rsp>
where
    P: IpcRequestPublisher + Send,
    Req: serde::Serialize + Send + Sync + 'static,
    Rsp: serde::de::DeserializeOwned + Send + Sync + 'static,
{
    type LayerType = SerdeJsonRequestSerializationPublisher<Req, Rsp, P>;

    fn layer(&self, publisher: P) -> Self::LayerType {
        SerdeJsonRequestSerializationPublisher {
            publisher,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct SerdeJsonRequestSerializationPublisher<Req, Rsp, P>
where
    P: IpcRequestPublisher,
{
    publisher: P,
    _phantom: std::marker::PhantomData<(Req, Rsp)>,
}

impl<Req, Rsp, P> Publisher for SerdeJsonRequestSerializationPublisher<Req, Rsp, P>
where
    P: IpcRequestPublisher + Sync,
    Req: Debug + serde::Serialize + Send + Sync + 'static,
    Rsp: Debug + serde::de::DeserializeOwned + Send + Sync + 'static,
{
    type Message = RequestImpl<Req, Rsp>;

    fn get_name(&self) -> &'static str {
        self.publisher.get_name()
    }

    fn publish(&self, message: Self::Message) -> BoxFuture<Result<()>> {
        let RequestImpl {
            content,
            response_sender,
        } = message;

        async move {
            let serialized_message = serde_json::to_string(&content)?;
            self.publisher
                .publish_request(
                    serialized_message.into(),
                    move |response| -> std::result::Result<(), String> {
                        let deserialized_response = serde_json::from_slice(&response)
                            .map_err(|err| format!("Failed to deserialize response: {}", err))?;

                        response_sender
                            .send(deserialized_response)
                            .map_err(|e| format!("Failed to send response: {:?}", e))?;
                        Ok(())
                    },
                )
                .await?;
            Ok(())
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
