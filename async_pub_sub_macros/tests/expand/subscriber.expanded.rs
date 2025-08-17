#![allow(unused_imports)]
use async_pub_sub::Subscriber;
use async_pub_sub_macros::DeriveSubscriber;
struct TestSubscriberA<A: Subscriber> {
    subscriber_a: A,
}
impl<A: Subscriber> async_pub_sub::Subscriber for TestSubscriberA<A> {
    type InputMessage = <A as async_pub_sub::Subscriber>::InputMessage;
    type OutputMessage = <A as async_pub_sub::Subscriber>::OutputMessage;
    fn get_name(&self) -> &'static str {
        async_pub_sub::Subscriber::get_name(&self.subscriber_a)
    }
    fn subscribe_to<P, Input>(&mut self, publisher: &mut P) -> async_pub_sub::Result<()>
    where
        P: async_pub_sub::PublisherWrapper<Input, Self::InputMessage>,
        Input: Send + 'static,
    {
        async_pub_sub::Subscriber::subscribe_to(&mut self.subscriber_a, publisher)
    }
    fn receive(
        &mut self,
    ) -> async_pub_sub::futures::future::BoxFuture<'_, Self::OutputMessage> {
        async_pub_sub::Subscriber::receive(&mut self.subscriber_a)
    }
}
struct TestSubscriberB<B>
where
    B: Subscriber,
{
    subscriber_b: B,
}
impl<B> async_pub_sub::Subscriber for TestSubscriberB<B>
where
    B: Subscriber,
{
    type InputMessage = <B as async_pub_sub::Subscriber>::InputMessage;
    type OutputMessage = <B as async_pub_sub::Subscriber>::OutputMessage;
    fn get_name(&self) -> &'static str {
        async_pub_sub::Subscriber::get_name(&self.subscriber_b)
    }
    fn subscribe_to<P, Input>(&mut self, publisher: &mut P) -> async_pub_sub::Result<()>
    where
        P: async_pub_sub::PublisherWrapper<Input, Self::InputMessage>,
        Input: Send + 'static,
    {
        async_pub_sub::Subscriber::subscribe_to(&mut self.subscriber_b, publisher)
    }
    fn receive(
        &mut self,
    ) -> async_pub_sub::futures::future::BoxFuture<'_, Self::OutputMessage> {
        async_pub_sub::Subscriber::receive(&mut self.subscriber_b)
    }
}
fn main() {}
