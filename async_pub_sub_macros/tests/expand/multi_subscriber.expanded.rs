use async_pub_sub::Subscriber;
use async_pub_sub_macros::DeriveSubscriber;
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
> async_pub_sub::SubscriberWrapper<<SubA as async_pub_sub::Subscriber>::Message>
for TestSubscriber<SubA, SubB>
where
    SubA: Subscriber,
    SubB: Subscriber,
{
    fn get_subscriber(
        &self,
    ) -> &impl async_pub_sub::Subscriber<
        Message = <SubA as async_pub_sub::Subscriber>::Message,
    > {
        &self.subscriber_a
    }
    fn get_subscriber_mut(
        &mut self,
    ) -> &mut impl async_pub_sub::Subscriber<
        Message = <SubA as async_pub_sub::Subscriber>::Message,
    > {
        &mut self.subscriber_a
    }
}
impl<
    SubA,
    SubB,
> async_pub_sub::SubscriberWrapper<<SubB as async_pub_sub::Subscriber>::Message>
for TestSubscriber<SubA, SubB>
where
    SubA: Subscriber,
    SubB: Subscriber,
{
    fn get_subscriber(
        &self,
    ) -> &impl async_pub_sub::Subscriber<
        Message = <SubB as async_pub_sub::Subscriber>::Message,
    > {
        &self.subscriber_b
    }
    fn get_subscriber_mut(
        &mut self,
    ) -> &mut impl async_pub_sub::Subscriber<
        Message = <SubB as async_pub_sub::Subscriber>::Message,
    > {
        &mut self.subscriber_b
    }
}
