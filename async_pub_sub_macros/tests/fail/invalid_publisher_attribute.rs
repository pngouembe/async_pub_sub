use async_pub_sub_macros::DerivePublisher;

#[derive(DerivePublisher)]
struct InvalidAttributePublisher {
    #[publisher(i32, String, bool)]  // Too many types in tuple
    field: String,
}

fn main() {}