use tokio_pub_sub::Subscriber;
use tokio_pub_sub_macros::DeriveSubscriber;
struct TestSubscriber<S: Subscriber> {
    test_subscriber: S,
}
impl<S: Subscriber> tokio_pub_sub::Subscriber for TestSubscriber<S> {
    type Message = <S as tokio_pub_sub::Subscriber>::Message;
    fn get_name(&self) -> &'static str {
        self.test_subscriber.get_name()
    }
    fn subscribe_to(
        &mut self,
        publisher: &mut impl tokio_pub_sub::Publisher<Message = Self::Message>,
    ) -> tokio_pub_sub::Result<()> {
        self.test_subscriber.subscribe_to(publisher)
    }
    fn receive(&mut self) -> impl std::future::Future<Output = Self::Message> {
        self.test_subscriber.receive()
    }
}
