use async_pub_sub_macros::DeriveSubscriber;

#[derive(DeriveSubscriber)]
struct InvalidAttributeSubscriber {
    #[subscriber(i32, String, bool)]  // Too many types in tuple
    field: String,
}

fn main() {}