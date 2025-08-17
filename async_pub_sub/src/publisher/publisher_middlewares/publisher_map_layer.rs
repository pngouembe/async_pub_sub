use std::pin::Pin;

use crate::{Layer, Publisher, Result};
use futures::future::BoxFuture;
use futures::{FutureExt, Stream};

pub struct PublisherMapLayer<InputMsg, OutputMsg> {
    map_function: Box<dyn Fn(InputMsg) -> Result<OutputMsg> + Send + Sync>,
}

impl<InputMsg, OutputMsg> PublisherMapLayer<InputMsg, OutputMsg> {
    pub fn new<F>(map_function: F) -> Self
    where
        F: Fn(InputMsg) -> Result<OutputMsg> + Send + Sync + 'static,
    {
        Self {
            map_function: Box::new(map_function),
        }
    }
}

impl<InputMsg, OutputMsg, P> Layer<P> for PublisherMapLayer<InputMsg, OutputMsg>
where
    P: Publisher<InputMessage = OutputMsg> + Send,
    InputMsg: Send + Sync + 'static,
    OutputMsg: Send + Sync + 'static,
{
    type LayerType = MappedPublisher<InputMsg, OutputMsg, P>;

    fn layer(self, publisher: P) -> Self::LayerType {
        MappedPublisher {
            map_function: self.map_function,
            publisher,
        }
    }
}

pub struct MappedPublisher<InputMsg, OutputMsg, P>
where
    P: Publisher<InputMessage = OutputMsg>,
{
    map_function: Box<dyn Fn(InputMsg) -> Result<OutputMsg> + Send + Sync>,
    publisher: P,
}

impl<InputMsg, OutputMsg, P> Publisher for MappedPublisher<InputMsg, OutputMsg, P>
where
    P: Publisher<InputMessage = OutputMsg> + Sync,
    InputMsg: Send + Sync + 'static,
    OutputMsg: Send + Sync + 'static,
{
    type InputMessage = InputMsg;
    type OutputMessage = P::OutputMessage;

    fn get_name(&self) -> &'static str {
        self.publisher.get_name()
    }

    fn publish(&self, message: Self::InputMessage) -> BoxFuture<'_, Result<()>> {
        async move {
            let mapped_message = (self.map_function)(message)?;
            self.publisher.publish(mapped_message).await
        }
        .boxed()
    }

    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<Pin<Box<dyn Stream<Item = Self::OutputMessage> + Send + Sync + 'static>>> {
        self.publisher.get_message_stream(subscriber_name)
    }
}
