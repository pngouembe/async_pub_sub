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
        Ok(())
            .and_then(|_| {
                async_pub_sub::SubscriberWrapper::<
                    _,
                >::subscribe_to(
                    &mut self.subscriber,
                    async_pub_sub::PublisherWrapper::<
                        _,
                    >::get_publisher_mut(&mut self.publisher1),
                )
            })
            .and_then(|_| {
                async_pub_sub::SubscriberWrapper::<
                    i32,
                >::subscribe_to(
                    &mut self.subscriber,
                    async_pub_sub::PublisherWrapper::<
                        _,
                    >::get_publisher_mut(&mut self.publisher2),
                )
            })
            .unwrap();
    }
}
fn main() {
    let mut publisher_a = PublisherImpl::<i32>::new("publisher_a", 1);
    let mut publisher_b = PublisherImpl::<i32>::new("publisher_b", 1);
    let mut publisher_c = PublisherImpl::<i32>::new("publisher_c", 1);
    let mut publisher_d = PublisherImpl::<i32>::new("publisher_d", 1);
    let mut subscriber = SubscriberImpl::<i32>::new("subscriber");
    async_pub_sub::SubscriberWrapper::<
        _,
    >::subscribe_to(
            &mut subscriber,
            async_pub_sub::PublisherWrapper::<_>::get_publisher_mut(&mut publisher_a),
        )
        .unwrap();
    async_pub_sub::SubscriberWrapper::<
        i32,
    >::subscribe_to(
            &mut subscriber,
            async_pub_sub::PublisherWrapper::<_>::get_publisher_mut(&mut publisher_b),
        )
        .unwrap();
    Ok(())
        .and_then(|_| {
            async_pub_sub::SubscriberWrapper::<
                _,
            >::subscribe_to(
                &mut subscriber,
                async_pub_sub::PublisherWrapper::<_>::get_publisher_mut(&mut publisher_c),
            )
        })
        .and_then(|_| {
            async_pub_sub::SubscriberWrapper::<
                i32,
            >::subscribe_to(
                &mut subscriber,
                async_pub_sub::PublisherWrapper::<_>::get_publisher_mut(&mut publisher_d),
            )
        })
        .unwrap();
}
