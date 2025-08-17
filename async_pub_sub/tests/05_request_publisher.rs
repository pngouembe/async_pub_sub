use async_pub_sub::{PublisherImpl, Request, RequestImpl, Result, SubscriberImpl};

#[test_log::test(tokio::test)]
async fn test_request_publisher() -> Result<()> {
    // -- Setup & Fixtures
    let mut subscriber = SubscriberImpl::<RequestImpl<i32, i32>>::new("subscriber");
    let mut publisher = PublisherImpl::new("publisher", 10);

    subscriber.subscribe_to(&mut publisher)?;

    // -- Exec
    let publisher_task = tokio::spawn(async move {
        let mut request = RequestImpl::new(42);
        let response = request.take_response().unwrap();
        publisher
            .publish(request)
            .await
            .expect("request published successfully");
        assert_eq!(response.await.expect("request successul"), 43);
    });

    let subscriber_task = tokio::spawn(async move {
        let request = subscriber.receive().await;
        let response = request.content.unwrap() + 1;

        request.respond(response).await.unwrap();
    });

    // -- Check
    tokio::try_join!(publisher_task, subscriber_task)?;

    Ok(())
}
