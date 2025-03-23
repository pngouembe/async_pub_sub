use async_pub_sub::SimplePublisher;
use async_pub_sub_macros::DerivePublisher;

#[derive(DerivePublisher)]
struct TestPublisherA {
    #[publisher(i32)]
    publisher_a: SimplePublisher<i32>,
}

#[derive(DerivePublisher)]
struct TestPublisherB {
    #[publisher(String)]
    publisher_b: SimplePublisher<String>,
}
