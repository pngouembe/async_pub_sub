use async_pub_sub::Publisher;
use async_pub_sub_macros::DerivePublisher;
struct TestPublisher<PubA, PubB>
where
    PubA: Publisher,
    PubB: Publisher,
{
    publisher_a: PubA,
    publisher_b: PubB,
}
impl<
    PubA,
    PubB,
> async_pub_sub::MultiPublisher<<PubA as async_pub_sub::Publisher>::Message>
for TestPublisher<PubA, PubB>
where
    PubA: Publisher,
    PubB: Publisher,
{
    fn get_publisher(
        &self,
    ) -> &impl async_pub_sub::Publisher<
        Message = <PubA as async_pub_sub::Publisher>::Message,
    > {
        &self.publisher_a
    }
    fn get_publisher_mut(
        &mut self,
    ) -> &mut impl async_pub_sub::Publisher<
        Message = <PubA as async_pub_sub::Publisher>::Message,
    > {
        &mut self.publisher_a
    }
}
impl<
    PubA,
    PubB,
> async_pub_sub::MultiPublisher<<PubB as async_pub_sub::Publisher>::Message>
for TestPublisher<PubA, PubB>
where
    PubA: Publisher,
    PubB: Publisher,
{
    fn get_publisher(
        &self,
    ) -> &impl async_pub_sub::Publisher<
        Message = <PubB as async_pub_sub::Publisher>::Message,
    > {
        &self.publisher_b
    }
    fn get_publisher_mut(
        &mut self,
    ) -> &mut impl async_pub_sub::Publisher<
        Message = <PubB as async_pub_sub::Publisher>::Message,
    > {
        &mut self.publisher_b
    }
}
