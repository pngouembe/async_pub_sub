use tokio_pub_sub::{LoggingPublisher, Publisher, Result, SimpleSubscriber};

#[test_log::test(tokio::test)]
async fn test_multiple_publishers() -> Result<()> {
    // -- Setup & Fixtures
    let mut publisher1 = LoggingPublisher::new("publisher1", 1);
    let mut publisher2 = LoggingPublisher::new("publisher2", 1);
    let mut subscriber = SimpleSubscriber::new("subscriber");

    subscriber.subscribe_to(&mut publisher1)?;
    subscriber.subscribe_to(&mut publisher2)?;

    // -- Exec
    publisher1.publish("Hello, publisher1").await?;
    publisher2.publish("Hello, publisher2").await?;

    let message1 = subscriber.receive().await;
    let message2 = subscriber.receive().await;

    // -- Check
    assert_eq!(message1, "Hello, publisher1");
    assert_eq!(message2, "Hello, publisher2");

    Ok(())
}
