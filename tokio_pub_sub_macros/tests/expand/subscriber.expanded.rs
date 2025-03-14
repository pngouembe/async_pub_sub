use tokio_pub_sub::Subscriber;
use tokio_pub_sub_macros::DeriveSubscriber;
struct TestSubscriberA<A: Subscriber> {
    subscriber_a: A,
}
impl<A: Subscriber> tokio_pub_sub::Subscriber for TestSubscriberA<A> {
    type Message = <A as tokio_pub_sub::Subscriber>::Message;
    fn get_name(&self) -> &'static str {
        self.subscriber_a.get_name()
    }
    fn subscribe_to(
        &mut self,
        publisher: &mut impl tokio_pub_sub::Publisher<Message = Self::Message>,
    ) -> tokio_pub_sub::Result<()> {
        self.subscriber_a.subscribe_to(publisher)
    }
    fn receive(&mut self) -> impl std::future::Future<Output = Self::Message> {
        self.subscriber_a.receive()
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
        self.subscriber_b.get_name()
    }
    fn subscribe_to(
        &mut self,
        publisher: &mut impl tokio_pub_sub::Publisher<Message = Self::Message>,
    ) -> tokio_pub_sub::Result<()> {
        self.subscriber_b.subscribe_to(publisher)
    }
    fn receive(&mut self) -> impl std::future::Future<Output = Self::Message> {
        self.subscriber_b.receive()
    }
}
