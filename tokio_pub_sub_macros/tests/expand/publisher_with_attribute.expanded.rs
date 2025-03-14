use tokio_pub_sub::SimplePublisher;
use tokio_pub_sub_macros::DerivePublisher;
struct TestPublisherA {
    #[publisher(i32)]
    publisher_a: SimplePublisher<i32>,
}
impl tokio_pub_sub::Publisher for TestPublisherA {
    type Message = i32;
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
struct TestPublisherB {
    #[publisher(String)]
    publisher_b: SimplePublisher<String>,
}
impl tokio_pub_sub::Publisher for TestPublisherB {
    type Message = String;
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
