use async_pub_sub::{PublisherImpl, Request, RequestImpl, Requester, Result, SubscriberImpl};

#[test_log::test(tokio::test)]
async fn test_custom_requests_publisher() -> Result<()> {
    // -- Setup & Fixtures
    let mut subscriber = SubscriberImpl::<RequestImpl<i32, i32>>::new("subscriber");
    let mut publisher = PublisherImpl::new("publisher", 10);

    subscriber.subscribe_to(&mut publisher)?;

    // -- Exec
    let publisher_task = tokio::spawn(async move {
        let request = RequestImpl::new(42);
        let response = publisher
            .request(request)
            .await
            .expect("request published successfully");

        assert_eq!(response, 43);
    });

    let subscriber_task = tokio::spawn(async move {
        let request = subscriber.receive().await;
        let response = request.content.unwrap() + 1;

        request
            .respond(response)
            .await
            .expect("response sent successfully");
    });

    // -- Check
    tokio::try_join!(publisher_task, subscriber_task)?;

    Ok(())
}
