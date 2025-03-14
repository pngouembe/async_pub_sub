use tokio_pub_sub::Publisher;
use tokio_pub_sub_macros::DerivePublisher;
struct TestPublisherA<P: Publisher> {
    publisher_a: P,
}
impl<P: Publisher> tokio_pub_sub::Publisher for TestPublisherA<P> {
    type Message = <P as tokio_pub_sub::Publisher>::Message;
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
struct TestPublisherB<P>
where
    P: Publisher,
{
    publisher_b: P,
}
impl<P> tokio_pub_sub::Publisher for TestPublisherB<P>
where
    P: Publisher,
{
    type Message = <P as tokio_pub_sub::Publisher>::Message;
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
