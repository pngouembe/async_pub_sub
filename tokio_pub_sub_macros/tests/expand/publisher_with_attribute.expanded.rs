use tokio_pub_sub::SimplePublisher;
use tokio_pub_sub_macros::DerivePublisher;
struct TestPublisherA {
    #[publisher(i32)]
    publisher_a: SimplePublisher<i32>,
}
impl tokio_pub_sub::Publisher for TestPublisherA {
    type Message = i32;
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
struct TestPublisherB {
    #[publisher(String)]
    publisher_b: SimplePublisher<String>,
}
impl tokio_pub_sub::Publisher for TestPublisherB {
    type Message = String;
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
