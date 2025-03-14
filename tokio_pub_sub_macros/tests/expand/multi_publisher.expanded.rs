use tokio_pub_sub::Publisher;
use tokio_pub_sub_macros::DerivePublisher;
struct TestPublisher<PubA, PubB>
where
    PubA: Publisher,
    PubB: Publisher,
{
    publisher_a: PubA,
    publisher_b: PubB,
}
impl<PubA, PubB> tokio_pub_sub::Publisher for TestPublisher<PubA, PubB>
where
    PubA: Publisher,
    PubB: Publisher,
{
    type Message = <PubA as tokio_pub_sub::Publisher>::Message;
    fn get_name(&self) -> &'static str {
        self.publisher_a.get_name()
    }
    fn publish_event(
        &self,
        message: Self::Message,
    ) -> futures::future::BoxFuture<tokio_pub_sub::Result<()>> {
        self.publisher_a.publish_event(message)
    }
    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> tokio_pub_sub::Result<
        std::pin::Pin<
            Box<dyn futures::Stream<Item = Self::Message> + Send + Sync + 'static>,
        >,
    > {
        self.publisher_a.get_message_stream(subscriber_name)
    }
}
impl<PubA, PubB> tokio_pub_sub::Publisher for TestPublisher<PubA, PubB>
where
    PubA: Publisher,
    PubB: Publisher,
{
    type Message = <PubB as tokio_pub_sub::Publisher>::Message;
    fn get_name(&self) -> &'static str {
        self.publisher_b.get_name()
    }
    fn publish_event(
        &self,
        message: Self::Message,
    ) -> futures::future::BoxFuture<tokio_pub_sub::Result<()>> {
        self.publisher_b.publish_event(message)
    }
    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> tokio_pub_sub::Result<
        std::pin::Pin<
            Box<dyn futures::Stream<Item = Self::Message> + Send + Sync + 'static>,
        >,
    > {
        self.publisher_b.get_message_stream(subscriber_name)
    }
}
