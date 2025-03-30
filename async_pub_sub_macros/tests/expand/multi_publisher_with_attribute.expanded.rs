use async_pub_sub::PublisherImpl;
use async_pub_sub_macros::DerivePublisher;
struct TestPublisher {
    #[publisher(i32)]
    publisher_a: PublisherImpl<i32>,
    #[publisher(String)]
    publisher_b: PublisherImpl<String>,
}
impl async_pub_sub::PublisherWrapper<i32> for TestPublisher {
    fn get_publisher(&self) -> &impl async_pub_sub::Publisher<Message = i32> {
        &self.publisher_a
    }
    fn get_publisher_mut(
        &mut self,
    ) -> &mut impl async_pub_sub::Publisher<Message = i32> {
        &mut self.publisher_a
    }
}
impl async_pub_sub::PublisherWrapper<String> for TestPublisher {
    fn get_publisher(&self) -> &impl async_pub_sub::Publisher<Message = String> {
        &self.publisher_b
    }
    fn get_publisher_mut(
        &mut self,
    ) -> &mut impl async_pub_sub::Publisher<Message = String> {
        &mut self.publisher_b
    }
}
fn main() {}
