use tokio_pub_sub::{LoggingPublisher, Publisher, Request, Result, SimpleSubscriber};

#[test_log::test(tokio::test)]
async fn test_request_publisher() -> Result<()> {
    // -- Setup & Fixtures
    let mut subscriber = SimpleSubscriber::<Request<i32, i32>>::new("subscriber");
    let mut publisher = LoggingPublisher::new("publisher", 10);

    subscriber.subscribe_to(&mut publisher)?;

    // -- Exec
    let publisher_task = tokio::spawn(async move {
        let response = publisher.publish_request(42).await;
        assert_eq!(response.expect("request successul"), 43);
    });

    let subscriber_task = tokio::spawn(async move {
        let request = subscriber.receive().await;
        let response = request.content + 1;

        request.respond(response);
    });

    // -- Check
    tokio::try_join!(publisher_task, subscriber_task)?;

    Ok(())
}
