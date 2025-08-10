mod deserialization_middlewares;
mod error;
mod serialization_middlewares;
mod wire_publishers;
mod wire_subscribers;

use bytes::Bytes;
use futures::future::BoxFuture;
pub use wire_publishers::NatsPublisher;
pub use wire_subscribers::NatsSubscriber;

pub use deserialization_middlewares::{
    SerdeJsonDeserializationLayer, SerdeJsonRequestDeserializationLayer,
};
pub use serialization_middlewares::{
    SerdeJsonRequestSerializationLayer, SerdeJsonSerializationLayer,
};

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
    ) -> BoxFuture<(
        Bytes,
        impl FnOnce(Bytes) -> BoxFuture<'static, ()> + Send + 'static,
    )>;
}
