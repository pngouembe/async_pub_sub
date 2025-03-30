#![allow(unused_imports)]
use async_pub_sub::Subscriber;
use async_pub_sub_macros::DeriveSubscriber;

#[derive(DeriveSubscriber)]
struct TestSubscriber<SubA, SubB>
where
    SubA: Subscriber<Message = i32>,
    SubB: Subscriber<Message = String>,
{
    subscriber_a: SubA,
    subscriber_b: SubB,
}

#[derive(DeriveSubscriber)]
struct MultiSubscriber<A: Subscriber<Message = i32>, B: Subscriber<Message = String>> {
    subscriber_a: A,
    subscriber_b: B,
}
fn main() {}
