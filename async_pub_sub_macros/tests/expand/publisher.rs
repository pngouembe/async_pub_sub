#![allow(unused_imports)]
use async_pub_sub::Publisher;
use async_pub_sub_macros::DerivePublisher;

#[derive(DerivePublisher)]
struct TestPublisherA<P: Publisher> {
    publisher_a: P,
}

#[derive(DerivePublisher)]
struct TestPublisherB<P>
where
    P: Publisher,
{
    publisher_b: P,
}

fn main() {}
