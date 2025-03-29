use async_pub_sub::{PublisherImpl, Result, SubscriberImpl, SubscriberWrapper};
use async_pub_sub_macros::DeriveSubscriber;

#[derive(DeriveSubscriber)]
struct MultiSub {
    #[subscriber(i32)]
    subscriber_a: SubscriberImpl<i32>,
    #[subscriber(String)]
    subscriber_b: SubscriberImpl<String>,
}

impl MultiSub {
    fn new() -> Self {
        Self {
            subscriber_a: SubscriberImpl::new("subscriber"),
            subscriber_b: SubscriberImpl::new("subscriber"),
        }
    }
}

#[tokio::test]
async fn test_multi_sub() -> Result<()> {
    let mut subscriber = MultiSub::new();

    let mut publisher1 = PublisherImpl::<i32>::new("publisher1", 1);
    let mut publisher2 = PublisherImpl::<String>::new("publisher2", 1);

    subscriber.subscribe_to(&mut publisher1)?;
    subscriber.subscribe_to(&mut publisher2)?;

    publisher1.publish(42).await?;
    let message = subscriber.subscriber_a.receive().await;
    assert_eq!(42, message);

    publisher2.publish("toto".to_string()).await?;
    let message = subscriber.subscriber_b.receive().await;
    assert_eq!("toto", message);

    Ok(())
}
