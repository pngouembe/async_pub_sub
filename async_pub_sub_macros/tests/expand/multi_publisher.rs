use async_pub_sub::Publisher;
use async_pub_sub_macros::DerivePublisher;

#[derive(DerivePublisher)]
struct TestPublisher<PubA, PubB>
where
    PubA: Publisher<InputMessage = i32, OutputMessage = i32>,
    PubB: Publisher<InputMessage = String, OutputMessage = String>,
{
    publisher_a: PubA,
    publisher_b: PubB,
}

#[derive(DerivePublisher)]
struct MultiPublisher<A: Publisher<InputMessage = i32, OutputMessage = i32>, B: Publisher<InputMessage = String, OutputMessage = String>> {
    publisher_a: A,
    publisher_b: B,
}

fn main() {}
