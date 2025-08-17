use async_pub_sub_macros::DerivePublisher;

#[derive(DerivePublisher)]
enum InvalidPublisher {
    Variant,
}

fn main() {}