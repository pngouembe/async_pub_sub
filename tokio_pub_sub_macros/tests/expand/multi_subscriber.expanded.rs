use tokio_pub_sub::Subscriber;
use tokio_pub_sub_macros::DeriveSubscriber;
struct TestSubscriber<SubA, SubB>
where
    SubA: Subscriber,
    SubB: Subscriber,
{
    subscriber_a: SubA,
    subscriber_b: SubB,
}
impl<SubA, SubB> tokio_pub_sub::Subscriber for TestSubscriber<SubA, SubB>
where
    SubA: Subscriber,
    SubB: Subscriber,
{
    type Message = <SubA as tokio_pub_sub::Subscriber>::Message;
    fn get_name(&self) -> &'static str {
        self.subscriber_a.get_name()
    }
    fn subscribe_to(
        &mut self,
        publisher: &mut impl tokio_pub_sub::MultiPublisher<Self::Message>,
    ) -> tokio_pub_sub::Result<()> {
        self.subscriber_a.subscribe_to(publisher)
    }
    fn receive(&mut self) -> impl std::future::Future<Output = Self::Message> {
        self.subscriber_a.receive()
    }
}
impl<SubA, SubB> tokio_pub_sub::Subscriber for TestSubscriber<SubA, SubB>
where
    SubA: Subscriber,
    SubB: Subscriber,
{
    type Message = <SubB as tokio_pub_sub::Subscriber>::Message;
    fn get_name(&self) -> &'static str {
        self.subscriber_b.get_name()
    }
    fn subscribe_to(
        &mut self,
        publisher: &mut impl tokio_pub_sub::MultiPublisher<Self::Message>,
    ) -> tokio_pub_sub::Result<()> {
        self.subscriber_b.subscribe_to(publisher)
    }
    fn receive(&mut self) -> impl std::future::Future<Output = Self::Message> {
        self.subscriber_b.receive()
    }
}
