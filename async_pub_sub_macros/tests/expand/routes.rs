#![allow(unused_imports)]
use async_pub_sub::{PublisherImpl, SubscriberImpl};
use async_pub_sub_macros::{route, routes};

struct MyComponent {
    publisher1: PublisherImpl<i32>,
    publisher2: PublisherImpl<i32>,
    subscriber: SubscriberImpl<i32>,
}

impl MyComponent {
    fn new(name: &'static str) -> Self {
        let publisher1 = PublisherImpl::<i32>::new(name, 1);
        let publisher2 = PublisherImpl::<i32>::new(name, 1);
        let subscriber = SubscriberImpl::<i32>::new(name);

        Self {
            publisher1,
            publisher2,
            subscriber,
        }
    }

    fn init(&mut self) {
        routes! {
            self.publisher1 -> self.subscriber,
            self.publisher2 -> self.subscriber: i32,
        }
        .unwrap();
    }
}

fn main() {
    let mut publisher_a = PublisherImpl::<i32>::new("publisher_a", 1);
    let mut publisher_b = PublisherImpl::<i32>::new("publisher_b", 1);
    let mut publisher_c = PublisherImpl::<i32>::new("publisher_c", 1);
    let mut publisher_d = PublisherImpl::<i32>::new("publisher_d", 1);
    let mut subscriber = SubscriberImpl::<i32>::new("subscriber");

    route!(publisher_a -> subscriber).unwrap();
    route!(publisher_b -> subscriber: i32).unwrap();
    routes!(
        publisher_c -> subscriber,
        publisher_d -> subscriber: i32,
    )
    .unwrap();
}
