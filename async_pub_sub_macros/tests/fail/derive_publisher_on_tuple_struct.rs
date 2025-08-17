use async_pub_sub_macros::DerivePublisher;

#[derive(DerivePublisher)]
struct InvalidPublisher(String);

fn main() {}