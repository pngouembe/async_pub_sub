use tokio_pub_sub::SimplePublisher;
use tokio_pub_sub_macros::DerivePublisher;

#[derive(DerivePublisher)]
struct TestPublisher {
    #[publisher(i32)]
    publisher_a: SimplePublisher<i32>,
    #[publisher(String)]
    publisher_b: SimplePublisher<String>,
}
