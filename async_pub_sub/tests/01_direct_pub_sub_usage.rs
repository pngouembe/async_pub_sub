use async_pub_sub::Result;
use tokio_implementations::{publisher::mpsc::MpscPublisher, subscriber::mpsc::MpscSubscriber};

#[tokio::test]
async fn test_pub_sub_i32() -> Result<()> {
    // -- Setup & Fixtures
    let mut subscriber = MpscSubscriber::new("subscriber");
    let mut publisher = MpscPublisher::new("publisher", 10);

    subscriber.subscribe_to(&mut publisher)?;

    // -- Exec
    publisher.publish(42).await?;
    let message = subscriber.receive().await;

    // -- Check
    assert_eq!(message, 42);

    Ok(())
}

#[tokio::test]
async fn test_pub_sub_string() -> Result<()> {
    // -- Setup & Fixtures
    let mut subscriber = MpscSubscriber::new("subscriber");
    let mut publisher = MpscPublisher::new("publisher", 10);

    subscriber.subscribe_to(&mut publisher)?;

    // -- Exec
    publisher.publish("hello").await?;
    let message = subscriber.receive().await;

    // -- Check
    assert_eq!(message, "hello");

    Ok(())
}
