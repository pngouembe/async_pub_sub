use async_pub_sub::{Publisher, Subscriber};
use async_pub_sub_macros::{route, routes};
fn main() {
    let mut publisher_a = Publisher::<i32>::new("publisher_a", 1);
    let mut publisher_b = Publisher::<i32>::new("publisher_b", 1);
    let mut subscriber = Subscriber::<i32>::new("subscriber", 1);
    {
        use async_pub_sub::SubscriberWrapper;
        subscriber.subscribe_to(&mut publisher_a)
    };
    async_pub_sub::SubscriberWrapper::<
        i32,
    >::subscribe_to(&mut subscriber, &mut publisher_b);
    Ok(())
        .and_then(|_| {
            {
                use async_pub_sub::SubscriberWrapper;
                subscriber.subscribe_to(&mut publisher_a)
            }
        })
        .and_then(|_| {
            async_pub_sub::SubscriberWrapper::<
                i32,
            >::subscribe_to(&mut subscriber, &mut publisher_b)
        });
}
