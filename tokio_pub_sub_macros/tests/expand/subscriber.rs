use tokio_pub_sub::Subscriber;
use tokio_pub_sub_macros::DeriveSubscriber;

#[derive(DeriveSubscriber)]
struct TestSubscriber<S: Subscriber> {
    test_subscriber: S,
}
