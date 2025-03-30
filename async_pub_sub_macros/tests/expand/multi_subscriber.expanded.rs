#![allow(unused_imports)]
use async_pub_sub::Subscriber;
use async_pub_sub_macros::DeriveSubscriber;
struct TestSubscriber<SubA, SubB>
where
    SubA: Subscriber<Message = i32>,
    SubB: Subscriber<Message = String>,
{
    subscriber_a: SubA,
    subscriber_b: SubB,
}
impl<SubA, SubB> async_pub_sub::SubscriberWrapper<i32> for TestSubscriber<SubA, SubB>
where
    SubA: Subscriber<Message = i32>,
    SubB: Subscriber<Message = String>,
{
    fn get_subscriber(&self) -> &impl async_pub_sub::Subscriber<Message = i32> {
        &self.subscriber_a
    }
    fn get_subscriber_mut(
        &mut self,
    ) -> &mut impl async_pub_sub::Subscriber<Message = i32> {
        &mut self.subscriber_a
    }
}
impl<SubA, SubB> async_pub_sub::SubscriberWrapper<String> for TestSubscriber<SubA, SubB>
where
    SubA: Subscriber<Message = i32>,
    SubB: Subscriber<Message = String>,
{
    fn get_subscriber(&self) -> &impl async_pub_sub::Subscriber<Message = String> {
        &self.subscriber_b
    }
    fn get_subscriber_mut(
        &mut self,
    ) -> &mut impl async_pub_sub::Subscriber<Message = String> {
        &mut self.subscriber_b
    }
}
struct MultiSubscriber<A: Subscriber<Message = i32>, B: Subscriber<Message = String>> {
    subscriber_a: A,
    subscriber_b: B,
}
impl<
    A: Subscriber<Message = i32>,
    B: Subscriber<Message = String>,
> async_pub_sub::SubscriberWrapper<i32> for MultiSubscriber<A, B> {
    fn get_subscriber(&self) -> &impl async_pub_sub::Subscriber<Message = i32> {
        &self.subscriber_a
    }
    fn get_subscriber_mut(
        &mut self,
    ) -> &mut impl async_pub_sub::Subscriber<Message = i32> {
        &mut self.subscriber_a
    }
}
impl<
    A: Subscriber<Message = i32>,
    B: Subscriber<Message = String>,
> async_pub_sub::SubscriberWrapper<String> for MultiSubscriber<A, B> {
    fn get_subscriber(&self) -> &impl async_pub_sub::Subscriber<Message = String> {
        &self.subscriber_b
    }
    fn get_subscriber_mut(
        &mut self,
    ) -> &mut impl async_pub_sub::Subscriber<Message = String> {
        &mut self.subscriber_b
    }
}
fn main() {}
