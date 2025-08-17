use crate::{Layer, PublisherWrapper, Result, Subscriber};
use futures::FutureExt;
use futures::future::BoxFuture;

pub struct SubscriberMapLayer<InputMsg, OutputMsg> {
    map_function: Box<dyn Fn(InputMsg) -> Result<OutputMsg> + Send + Sync>,
}

impl<InputMsg, OutputMsg> SubscriberMapLayer<InputMsg, OutputMsg> {
    pub fn new<F>(map_function: F) -> Self
    where
        F: Fn(InputMsg) -> Result<OutputMsg> + Send + Sync + 'static,
    {
        Self {
            map_function: Box::new(map_function),
        }
    }
}

impl<InputMsg, OutputMsg, S> Layer<S> for SubscriberMapLayer<InputMsg, OutputMsg>
where
    S: Subscriber<OutputMessage = InputMsg>,
{
    type LayerType = SubscriberMapSubscriber<OutputMsg, S>;

    fn layer(self, subscriber: S) -> Self::LayerType {
        SubscriberMapSubscriber {
            map_function: self.map_function,
            subscriber,
        }
    }
}

pub struct SubscriberMapSubscriber<Message, S>
where
    S: Subscriber,
{
    map_function: Box<dyn Fn(S::OutputMessage) -> Result<Message> + Send + Sync>,
    subscriber: S,
}

impl<Message, S> Subscriber for SubscriberMapSubscriber<Message, S>
where
    S: Subscriber,
    Message: Send + 'static,
{
    type InputMessage = S::InputMessage;
    type OutputMessage = Message;

    fn get_name(&self) -> &'static str {
        self.subscriber.get_name()
    }

    fn subscribe_to<P, Input>(&mut self, publisher: &mut P) -> Result<()>
    where
        P: PublisherWrapper<Input, Self::InputMessage>,
        Input: Send + 'static,
    {
        self.subscriber.subscribe_to(publisher)
    }

    fn receive(&mut self) -> BoxFuture<'_, Self::OutputMessage> {
        let future_message = self.subscriber.receive();
        let map_function = &self.map_function;

        async move {
            let input_message = future_message.await;
            map_function(input_message).expect("Message mapping should not fail")
        }
        .boxed()
    }
}
