use tokio_pub_sub::Subscriber;
use tokio_pub_sub_macros::DeriveSubscriber;

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
