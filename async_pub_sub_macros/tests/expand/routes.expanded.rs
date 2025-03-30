#![allow(unused_imports)]
use async_pub_sub::{PublisherImpl, SubscriberImpl};
use async_pub_sub_macros::{route, routes};
fn main() {
    let mut publisher_a = PublisherImpl::<i32>::new("publisher_a", 1);
    let mut publisher_b = PublisherImpl::<i32>::new("publisher_b", 1);
    let mut publisher_c = PublisherImpl::<i32>::new("publisher_c", 1);
    let mut publisher_d = PublisherImpl::<i32>::new("publisher_d", 1);
    let mut subscriber = SubscriberImpl::<i32>::new("subscriber");
    {
        use async_pub_sub::SubscriberWrapper;
        subscriber.subscribe_to(&mut publisher_a)
    }
        .unwrap();
    async_pub_sub::SubscriberWrapper::<
        i32,
    >::subscribe_to(&mut subscriber, &mut publisher_b)
        .unwrap();
    Ok(())
        .and_then(|_| {
            {
                use async_pub_sub::SubscriberWrapper;
                subscriber.subscribe_to(&mut publisher_c)
            }
        })
        .and_then(|_| {
            async_pub_sub::SubscriberWrapper::<
                i32,
            >::subscribe_to(&mut subscriber, &mut publisher_d)
        })
        .unwrap();
}
