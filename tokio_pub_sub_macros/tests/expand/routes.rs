use tokio_pub_sub::{Publisher, Subscriber};
use tokio_pub_sub_macros::route;

fn main() {
    let mut publisher_a = Publisher::<i32>::new("publisher_a", 1);
    let mut publisher_b = Publisher::<i32>::new("publisher_b", 1);
    let mut subscriber = Subscriber::<i32>::new("subscriber", 1);

    route!(subscriber, publisher_a);
    route!(subscriber, publisher_b: i32);
}
