mod deserialization_middlewares;
mod error;
mod serialization_middlewares;
mod wire_publishers;
mod wire_subscribers;

use bytes::Bytes;
use futures::future::BoxFuture;
pub use wire_publishers::{NatsPublisher, NatsRequestPublisher};
pub use wire_subscribers::{NatsRequest, NatsRequestSubscriber, NatsSubscriber};

pub use deserialization_middlewares::{
    SerdeJsonDeserializationLayer, SerdeJsonRequestDeserializationLayer,
    SerdeRequestDeserializationLayer,
};
pub use serialization_middlewares::{SerdeJsonSerializationLayer, SerdeRequestSerializationLayer};

pub use error::{Error, Result};

pub trait IpcRequestPublisher {
    fn get_name(&self) -> &'static str;
    fn publish_request(
        &self,
        request: Bytes,
        response_callback: impl FnOnce(Bytes) -> std::result::Result<(), String> + Send + 'static,
    ) -> impl Future<Output = Result<()>> + Send;
}

pub trait IpcRequestSubscriber {
    fn get_name(&self) -> &'static str;
    fn receive_request(
        &mut self,
    ) -> BoxFuture<
        '_,
        (
            Bytes,
            impl FnOnce(Bytes) -> BoxFuture<'static, ()> + Send + 'static,
        ),
    >;
}
