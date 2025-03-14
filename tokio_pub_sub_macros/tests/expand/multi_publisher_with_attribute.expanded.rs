use tokio_pub_sub::SimplePublisher;
use tokio_pub_sub_macros::DerivePublisher;
struct TestPublisher {
    #[publisher(i32)]
    publisher_a: SimplePublisher<i32>,
    #[publisher(String)]
    publisher_b: SimplePublisher<String>,
}
impl tokio_pub_sub::MultiPublisher<i32> for TestPublisher {
    fn get_publisher(&self) -> &impl tokio_pub_sub::Publisher<Message = i32> {
        &self.publisher_a
    }
    fn get_publisher_mut(
        &mut self,
    ) -> &mut impl tokio_pub_sub::Publisher<Message = i32> {
        &mut self.publisher_a
    }
}
impl tokio_pub_sub::MultiPublisher<String> for TestPublisher {
    fn get_publisher(&self) -> &impl tokio_pub_sub::Publisher<Message = String> {
        &self.publisher_b
    }
    fn get_publisher_mut(
        &mut self,
    ) -> &mut impl tokio_pub_sub::Publisher<Message = String> {
        &mut self.publisher_b
    }
}
