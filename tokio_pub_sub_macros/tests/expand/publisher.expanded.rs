use tokio_pub_sub::Publisher;
use tokio_pub_sub_macros::DerivePublisher;
struct TestPublisherA<P: Publisher> {
    publisher_a: P,
}
impl<P: Publisher> tokio_pub_sub::Publisher for TestPublisherA<P> {
    type Message = <P as tokio_pub_sub::Publisher>::Message;
    fn get_name(&self) -> &'static str {
        tokio_pub_sub::Publisher::get_name(&self.publisher_a)
    }
    fn publish(
        &self,
        message: Self::Message,
    ) -> futures::future::BoxFuture<tokio_pub_sub::Result<()>> {
        tokio_pub_sub::Publisher::publish(&self.publisher_a, message)
    }
    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> tokio_pub_sub::Result<
        std::pin::Pin<
            Box<dyn futures::Stream<Item = Self::Message> + Send + Sync + 'static>,
        >,
    > {
        tokio_pub_sub::Publisher::get_message_stream(
            &mut self.publisher_a,
            subscriber_name,
        )
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
        tokio_pub_sub::Publisher::get_name(&self.publisher_b)
    }
    fn publish(
        &self,
        message: Self::Message,
    ) -> futures::future::BoxFuture<tokio_pub_sub::Result<()>> {
        tokio_pub_sub::Publisher::publish(&self.publisher_b, message)
    }
    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> tokio_pub_sub::Result<
        std::pin::Pin<
            Box<dyn futures::Stream<Item = Self::Message> + Send + Sync + 'static>,
        >,
    > {
        tokio_pub_sub::Publisher::get_message_stream(
            &mut self.publisher_b,
            subscriber_name,
        )
    }
}
