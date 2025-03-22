use tokio_pub_sub::{Publisher, Subscriber};
use tokio_pub_sub_macros::{route, routes};

fn main() {
    let mut publisher_a = Publisher::<i32>::new("publisher_a", 1);
    let mut publisher_b = Publisher::<i32>::new("publisher_b", 1);
    let mut subscriber = Subscriber::<i32>::new("subscriber", 1);

    route!(publisher_a -> subscriber);
    route!(publisher_b -> subscriber: i32);
    routes!(
        publisher_a -> subscriber,
        publisher_b -> subscriber: i32,
    );
}
