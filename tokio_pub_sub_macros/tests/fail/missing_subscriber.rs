use tokio_pub_sub_macros::DeriveSubscriber;

#[derive(DeriveSubscriber)]
struct InvalidSubscriber {
    other_field: String,
}

fn main() {}
