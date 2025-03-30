use async_pub_sub::Publisher;
use async_pub_sub_macros::DerivePublisher;
struct TestPublisher<PubA, PubB>
where
    PubA: Publisher<Message = i32>,
    PubB: Publisher<Message = String>,
{
    publisher_a: PubA,
    publisher_b: PubB,
}
impl<PubA, PubB> async_pub_sub::PublisherWrapper<i32> for TestPublisher<PubA, PubB>
where
    PubA: Publisher<Message = i32>,
    PubB: Publisher<Message = String>,
{
    fn get_publisher(&self) -> &impl async_pub_sub::Publisher<Message = i32> {
        &self.publisher_a
    }
    fn get_publisher_mut(
        &mut self,
    ) -> &mut impl async_pub_sub::Publisher<Message = i32> {
        &mut self.publisher_a
    }
}
impl<PubA, PubB> async_pub_sub::PublisherWrapper<String> for TestPublisher<PubA, PubB>
where
    PubA: Publisher<Message = i32>,
    PubB: Publisher<Message = String>,
{
    fn get_publisher(&self) -> &impl async_pub_sub::Publisher<Message = String> {
        &self.publisher_b
    }
    fn get_publisher_mut(
        &mut self,
    ) -> &mut impl async_pub_sub::Publisher<Message = String> {
        &mut self.publisher_b
    }
}
struct MultiPublisher<A: Publisher<Message = i32>, B: Publisher<Message = String>> {
    publisher_a: A,
    publisher_b: B,
}
impl<
    A: Publisher<Message = i32>,
    B: Publisher<Message = String>,
> async_pub_sub::PublisherWrapper<i32> for MultiPublisher<A, B> {
    fn get_publisher(&self) -> &impl async_pub_sub::Publisher<Message = i32> {
        &self.publisher_a
    }
    fn get_publisher_mut(
        &mut self,
    ) -> &mut impl async_pub_sub::Publisher<Message = i32> {
        &mut self.publisher_a
    }
}
impl<
    A: Publisher<Message = i32>,
    B: Publisher<Message = String>,
> async_pub_sub::PublisherWrapper<String> for MultiPublisher<A, B> {
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
