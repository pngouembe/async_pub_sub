use std::{fmt::Debug, pin::Pin};

use futures::{future::BoxFuture, FutureExt, Stream};

use crate::{Publisher, PublisherLayer, Result};

pub struct DebugingPublisherLayer;

impl<P> PublisherLayer<P> for DebugingPublisherLayer
where
    P: Publisher + Send + Sync,
    P::Message: Debug,
{
    type PublisherType = DebugPublisher<P>;

    fn layer(&self, publisher: P) -> Self::PublisherType {
        DebugPublisher {
            subscriber_name: None,
            publisher,
        }
    }
}

pub struct DebugPublisher<P>
where
    P: Publisher,
{
    subscriber_name: Option<&'static str>,
    publisher: P,
}

impl<P> Publisher for DebugPublisher<P>
where
    P: Publisher,
    P::Message: Debug,
    Self: Sync,
{
    type Message = P::Message;

    fn get_name(&self) -> &'static str {
        self.publisher.get_name()
    }

    fn publish_event(&self, message: Self::Message) -> BoxFuture<Result<()>> {
        async move {
            let message_str = format!("{:?}", &message);
            let result = self.publisher.publish_event(message).await;
            log::info!(
                "[{}] -> [{}]: {}",
                self.publisher.get_name(),
                self.subscriber_name
                    .expect("subscriber name should be known"),
                message_str
            );
            result
        }
        .boxed()
    }

    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<Pin<Box<dyn Stream<Item = Self::Message> + Send + Sync + 'static>>> {
        self.subscriber_name = Some(subscriber_name);
        self.publisher.get_message_stream(subscriber_name)
    }
}
