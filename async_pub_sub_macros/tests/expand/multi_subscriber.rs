use async_pub_sub::Subscriber;
use async_pub_sub_macros::DeriveSubscriber;

#[derive(DeriveSubscriber)]
struct TestSubscriber<SubA, SubB>
where
    SubA: Subscriber,
    SubB: Subscriber,
{
    subscriber_a: SubA,
    subscriber_b: SubB,
}
