use async_pub_sub::Publisher;
use async_pub_sub_macros::DerivePublisher;

#[derive(DerivePublisher)]
struct TestPublisher<PubA, PubB>
where
    PubA: Publisher<Message = i32>,
    PubB: Publisher<Message = String>,
{
    publisher_a: PubA,
    publisher_b: PubB,
}

#[derive(DerivePublisher)]
struct MultiPublisher<A: Publisher<Message = i32>, B: Publisher<Message = String>> {
    publisher_a: A,
    publisher_b: B,
}

fn main() {}
