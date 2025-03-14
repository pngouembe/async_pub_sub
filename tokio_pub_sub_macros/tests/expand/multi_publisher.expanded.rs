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
impl<
    PubA,
    PubB,
> tokio_pub_sub::MultiPublisher<<PubA as tokio_pub_sub::Publisher>::Message>
for TestPublisher<PubA, PubB>
where
    PubA: Publisher,
    PubB: Publisher,
{
    fn get_publisher(
        &self,
    ) -> &impl tokio_pub_sub::Publisher<
        Message = <PubA as tokio_pub_sub::Publisher>::Message,
    > {
        &self.publisher_a
    }
    fn get_publisher_mut(
        &mut self,
    ) -> &mut impl tokio_pub_sub::Publisher<
        Message = <PubA as tokio_pub_sub::Publisher>::Message,
    > {
        &mut self.publisher_a
    }
}
impl<
    PubA,
    PubB,
> tokio_pub_sub::MultiPublisher<<PubB as tokio_pub_sub::Publisher>::Message>
for TestPublisher<PubA, PubB>
where
    PubA: Publisher,
    PubB: Publisher,
{
    fn get_publisher(
        &self,
    ) -> &impl tokio_pub_sub::Publisher<
        Message = <PubB as tokio_pub_sub::Publisher>::Message,
    > {
        &self.publisher_b
    }
    fn get_publisher_mut(
        &mut self,
    ) -> &mut impl tokio_pub_sub::Publisher<
        Message = <PubB as tokio_pub_sub::Publisher>::Message,
    > {
        &mut self.publisher_b
    }
}
