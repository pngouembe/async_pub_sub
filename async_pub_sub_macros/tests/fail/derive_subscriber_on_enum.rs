use async_pub_sub_macros::DeriveSubscriber;

#[derive(DeriveSubscriber)]
enum InvalidSubscriber {
    Toto,
}

fn main() {}
