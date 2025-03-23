use async_pub_sub::{MultiPublisher, Result};
use async_pub_sub_macros::DerivePublisher;
use tokio_implementations::{publisher::mpsc::MpscPublisher, subscriber::mpsc::MpscSubscriber};

#[derive(DerivePublisher)]
struct MultiPub {
    #[publisher(i32)]
    publisher_a: MpscPublisher<i32>,
    #[publisher(String)]
    publisher_b: MpscPublisher<String>,
}

impl MultiPub {
    fn new() -> Self {
        Self {
            publisher_a: MpscPublisher::new("publisher", 1),
            publisher_b: MpscPublisher::new("publisher", 1),
        }
    }
}

#[tokio::test]
async fn test_multi_pub() -> Result<()> {
    let mut subscriber1 = MpscSubscriber::<i32>::new("subscriber1");
    let mut subscriber2 = MpscSubscriber::<String>::new("subscriber2");

    let mut publisher = MultiPub::new();
    subscriber1.subscribe_to(&mut publisher)?;
    subscriber2.subscribe_to(&mut publisher)?;

    publisher.publish(42).await?;
    let message = subscriber1.receive().await;
    assert_eq!(message, 42);

    publisher.publish("toto".to_string()).await?;
    let message = subscriber2.receive().await;
    assert_eq!(message, "toto");

    Ok(())
}
