#![allow(unused_imports)]
use async_pub_sub::Subscriber;
use async_pub_sub_macros::DeriveSubscriber;

#[derive(DeriveSubscriber)]
struct TestSubscriber<SubA, SubB>
where
    SubA: Subscriber<InputMessage = i32, OutputMessage = i32>,
    SubB: Subscriber<InputMessage = String, OutputMessage = String>,
{
    subscriber_a: SubA,
    subscriber_b: SubB,
}

#[derive(DeriveSubscriber)]
struct MultiSubscriber<A: Subscriber<InputMessage = i32, OutputMessage = i32>, B: Subscriber<InputMessage = String, OutputMessage = String>> {
    subscriber_a: A,
    subscriber_b: B,
}
fn main() {}
