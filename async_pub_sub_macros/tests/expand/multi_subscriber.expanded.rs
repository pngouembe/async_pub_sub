#![allow(unused_imports)]
use async_pub_sub::Subscriber;
use async_pub_sub_macros::DeriveSubscriber;
struct TestSubscriber<SubA, SubB>
where
    SubA: Subscriber<InputMessage = i32, OutputMessage = i32>,
    SubB: Subscriber<InputMessage = String, OutputMessage = String>,
{
    subscriber_a: SubA,
    subscriber_b: SubB,
}
impl<SubA, SubB> async_pub_sub::SubscriberWrapper<i32, i32>
for TestSubscriber<SubA, SubB>
where
    SubA: Subscriber<InputMessage = i32, OutputMessage = i32>,
    SubB: Subscriber<InputMessage = String, OutputMessage = String>,
{
    fn get_subscriber(
        &self,
    ) -> &impl async_pub_sub::Subscriber<InputMessage = i32, OutputMessage = i32> {
        &self.subscriber_a
    }
    fn get_subscriber_mut(
        &mut self,
    ) -> &mut impl async_pub_sub::Subscriber<InputMessage = i32, OutputMessage = i32> {
        &mut self.subscriber_a
    }
}
impl<SubA, SubB> async_pub_sub::SubscriberWrapper<String, String>
for TestSubscriber<SubA, SubB>
where
    SubA: Subscriber<InputMessage = i32, OutputMessage = i32>,
    SubB: Subscriber<InputMessage = String, OutputMessage = String>,
{
    fn get_subscriber(
        &self,
    ) -> &impl async_pub_sub::Subscriber<InputMessage = String, OutputMessage = String> {
        &self.subscriber_b
    }
    fn get_subscriber_mut(
        &mut self,
    ) -> &mut impl async_pub_sub::Subscriber<
        InputMessage = String,
        OutputMessage = String,
    > {
        &mut self.subscriber_b
    }
}
struct MultiSubscriber<
    A: Subscriber<InputMessage = i32, OutputMessage = i32>,
    B: Subscriber<InputMessage = String, OutputMessage = String>,
> {
    subscriber_a: A,
    subscriber_b: B,
}
impl<
    A: Subscriber<InputMessage = i32, OutputMessage = i32>,
    B: Subscriber<InputMessage = String, OutputMessage = String>,
> async_pub_sub::SubscriberWrapper<i32, i32> for MultiSubscriber<A, B> {
    fn get_subscriber(
        &self,
    ) -> &impl async_pub_sub::Subscriber<InputMessage = i32, OutputMessage = i32> {
        &self.subscriber_a
    }
    fn get_subscriber_mut(
        &mut self,
    ) -> &mut impl async_pub_sub::Subscriber<InputMessage = i32, OutputMessage = i32> {
        &mut self.subscriber_a
    }
}
impl<
    A: Subscriber<InputMessage = i32, OutputMessage = i32>,
    B: Subscriber<InputMessage = String, OutputMessage = String>,
> async_pub_sub::SubscriberWrapper<String, String> for MultiSubscriber<A, B> {
    fn get_subscriber(
        &self,
    ) -> &impl async_pub_sub::Subscriber<InputMessage = String, OutputMessage = String> {
        &self.subscriber_b
    }
    fn get_subscriber_mut(
        &mut self,
    ) -> &mut impl async_pub_sub::Subscriber<
        InputMessage = String,
        OutputMessage = String,
    > {
        &mut self.subscriber_b
    }
}
fn main() {}
