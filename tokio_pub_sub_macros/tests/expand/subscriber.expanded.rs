use tokio_pub_sub::Subscriber;
use tokio_pub_sub_macros::DeriveSubscriber;
struct TestSubscriberA<A: Subscriber> {
    subscriber_a: A,
}
impl<A: Subscriber> tokio_pub_sub::Subscriber for TestSubscriberA<A> {
    type Message = <A as tokio_pub_sub::Subscriber>::Message;
    fn get_name(&self) -> &'static str {
        tokio_pub_sub::Subscriber::get_name(&self.subscriber_a)
    }
    fn subscribe_to(
        &mut self,
        publisher: &mut impl tokio_pub_sub::MultiPublisher<Self::Message>,
    ) -> tokio_pub_sub::Result<()> {
        tokio_pub_sub::Subscriber::subscribe_to(&mut self.subscriber_a, publisher)
    }
    fn receive(&mut self) -> impl std::future::Future<Output = Self::Message> {
        tokio_pub_sub::Subscriber::receive(&mut self.subscriber_a)
    }
}
struct TestSubscriberB<B>
where
    B: Subscriber,
{
    subscriber_b: B,
}
impl<B> tokio_pub_sub::Subscriber for TestSubscriberB<B>
where
    B: Subscriber,
{
    type Message = <B as tokio_pub_sub::Subscriber>::Message;
    fn get_name(&self) -> &'static str {
        tokio_pub_sub::Subscriber::get_name(&self.subscriber_b)
    }
    fn subscribe_to(
        &mut self,
        publisher: &mut impl tokio_pub_sub::MultiPublisher<Self::Message>,
    ) -> tokio_pub_sub::Result<()> {
        tokio_pub_sub::Subscriber::subscribe_to(&mut self.subscriber_b, publisher)
    }
    fn receive(&mut self) -> impl std::future::Future<Output = Self::Message> {
        tokio_pub_sub::Subscriber::receive(&mut self.subscriber_b)
    }
}
