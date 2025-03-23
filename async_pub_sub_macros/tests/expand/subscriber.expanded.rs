use async_pub_sub::Subscriber;
use async_pub_sub_macros::DeriveSubscriber;
struct TestSubscriberA<A: Subscriber> {
    subscriber_a: A,
}
impl<A: Subscriber> async_pub_sub::Subscriber for TestSubscriberA<A> {
    type Message = <A as async_pub_sub::Subscriber>::Message;
    fn get_name(&self) -> &'static str {
        async_pub_sub::Subscriber::get_name(&self.subscriber_a)
    }
    fn subscribe_to(
        &mut self,
        publisher: &mut impl async_pub_sub::MultiPublisher<Self::Message>,
    ) -> async_pub_sub::Result<()> {
        async_pub_sub::Subscriber::subscribe_to(&mut self.subscriber_a, publisher)
    }
    fn receive(&mut self) -> impl std::future::Future<Output = Self::Message> {
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
    type Message = <B as async_pub_sub::Subscriber>::Message;
    fn get_name(&self) -> &'static str {
        async_pub_sub::Subscriber::get_name(&self.subscriber_b)
    }
    fn subscribe_to(
        &mut self,
        publisher: &mut impl async_pub_sub::MultiPublisher<Self::Message>,
    ) -> async_pub_sub::Result<()> {
        async_pub_sub::Subscriber::subscribe_to(&mut self.subscriber_b, publisher)
    }
    fn receive(&mut self) -> impl std::future::Future<Output = Self::Message> {
        async_pub_sub::Subscriber::receive(&mut self.subscriber_b)
    }
}
