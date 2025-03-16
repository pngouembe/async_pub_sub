use tokio_pub_sub::Subscriber;
use tokio_pub_sub_macros::DeriveSubscriber;
struct TestSubscriber<SubA, SubB>
where
    SubA: Subscriber,
    SubB: Subscriber,
{
    subscriber_a: SubA,
    subscriber_b: SubB,
}
impl<
    SubA,
    SubB,
> tokio_pub_sub::MultiSubscriber<<SubA as tokio_pub_sub::Subscriber>::Message>
for TestSubscriber<SubA, SubB>
where
    SubA: Subscriber,
    SubB: Subscriber,
{
    fn get_subscriber(
        &self,
    ) -> &impl tokio_pub_sub::Subscriber<
        Message = <SubA as tokio_pub_sub::Subscriber>::Message,
    > {
        &self.subscriber_a
    }
    fn get_subscriber_mut(
        &mut self,
    ) -> &mut impl tokio_pub_sub::Subscriber<
        Message = <SubA as tokio_pub_sub::Subscriber>::Message,
    > {
        &mut self.subscriber_a
    }
}
impl<
    SubA,
    SubB,
> tokio_pub_sub::MultiSubscriber<<SubB as tokio_pub_sub::Subscriber>::Message>
for TestSubscriber<SubA, SubB>
where
    SubA: Subscriber,
    SubB: Subscriber,
{
    fn get_subscriber(
        &self,
    ) -> &impl tokio_pub_sub::Subscriber<
        Message = <SubB as tokio_pub_sub::Subscriber>::Message,
    > {
        &self.subscriber_b
    }
    fn get_subscriber_mut(
        &mut self,
    ) -> &mut impl tokio_pub_sub::Subscriber<
        Message = <SubB as tokio_pub_sub::Subscriber>::Message,
    > {
        &mut self.subscriber_b
    }
}