use async_pub_sub::PublisherImpl;
use async_pub_sub_macros::DerivePublisher;

#[derive(DerivePublisher)]
struct TestPublisher {
    #[publisher(i32)]
    publisher_a: PublisherImpl<i32>,
    #[publisher(String)]
    publisher_b: PublisherImpl<String>,
}

fn main() {}
