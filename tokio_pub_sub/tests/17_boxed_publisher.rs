use tokio_pub_sub::{Publisher, Result, SimplePublisher, SimpleSubscriber};

#[tokio::test]
async fn test_pub_sub_i32() -> Result<()> {
    // -- Setup & Fixtures
    let mut subscriber = SimpleSubscriber::new("subscriber");
    let mut publisher: Box<dyn Publisher<Message = i32>> =
        Box::new(SimplePublisher::new("publisher", 10));

    subscriber.subscribe_to(&mut publisher)?;

    // -- Exec
    publisher.publish_event(42).await?;
    let message = subscriber.receive().await;

    // -- Check
    assert_eq!(message, 42);

    Ok(())
}
