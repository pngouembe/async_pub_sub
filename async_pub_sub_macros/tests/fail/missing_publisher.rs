use async_pub_sub_macros::DerivePublisher;

#[derive(DerivePublisher)]
struct InvalidPublisher {
    other_field: String,
}

fn main() {}
