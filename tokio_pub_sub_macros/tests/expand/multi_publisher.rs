use tokio_pub_sub::Publisher;
use tokio_pub_sub_macros::DerivePublisher;

#[derive(DerivePublisher)]
struct TestPublisher<PubA, PubB>
where
    PubA: Publisher,
    PubB: Publisher,
{
    publisher_a: PubA,
    publisher_b: PubB,
}
