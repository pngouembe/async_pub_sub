#![allow(unused_imports)]
use async_pub_sub::PublisherImpl;
use async_pub_sub_macros::DerivePublisher;

#[derive(DerivePublisher)]
struct TestPublisherA {
    #[publisher(i32)]
    publisher_a: PublisherImpl<i32>,
}

#[derive(DerivePublisher)]
struct TestPublisherB {
    #[publisher(String)]
    publisher_b: PublisherImpl<String>,
}

fn main() {}
