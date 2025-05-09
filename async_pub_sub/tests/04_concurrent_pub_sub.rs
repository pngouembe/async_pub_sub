use async_pub_sub::{PublisherImpl, Result, SubscriberImpl};

#[test_log::test(tokio::test)]
async fn test_request_publisher() -> Result<()> {
    // -- Setup & Fixtures
    let mut subscriber = SubscriberImpl::new("subscriber");
    let mut publisher = PublisherImpl::new("publisher", 10);

    subscriber.subscribe_to(&mut publisher)?;

    // -- Exec
    let publisher_task = tokio::spawn(async move {
        let result = publisher.publish(42).await;
        assert!(result.is_ok());
    });

    let subscriber_task = tokio::spawn(async move {
        let message = subscriber.receive().await;
        assert_eq!(message, 42);
    });

    // -- Check
    tokio::try_join!(publisher_task, subscriber_task)?;

    Ok(())
}
