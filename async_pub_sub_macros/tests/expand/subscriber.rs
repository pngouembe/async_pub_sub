#![allow(unused_imports)]
use async_pub_sub::Subscriber;
use async_pub_sub_macros::DeriveSubscriber;

#[derive(DeriveSubscriber)]
struct TestSubscriberA<A: Subscriber> {
    subscriber_a: A,
}

#[derive(DeriveSubscriber)]
struct TestSubscriberB<B>
where
    B: Subscriber,
{
    subscriber_b: B,
}

fn main() {}
