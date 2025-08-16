use async_pub_sub::{
    LoggingRequestLayer, PublisherImpl, Request, RequestBuilder, RequestImpl, Requester, 
    ResponseTransformRequestLayer, Result, SubscriberImpl,
};

#[test_log::test(tokio::test)]
async fn test_request_middleware_with_response_transform() -> Result<()> {
    // -- Setup & Fixtures
    let mut subscriber = SubscriberImpl::new("subscriber");
    let mut publisher = PublisherImpl::new("publisher", 10);

    subscriber.subscribe_to(&mut publisher)?;

    // -- Exec
    let publisher_task = tokio::spawn(async move {
        // Create a request with response transformation: i32 -> String
        let base_request = RequestImpl::new(42);
        let transform_layer = ResponseTransformRequestLayer::new(|response: i32| {
            format!("Response: {}", response)
        });
        
        let transformed_request = RequestBuilder::new()
            .layer(transform_layer)
            .request(base_request);

        let response: String = publisher
            .request(transformed_request)
            .await
            .expect("request published successfully");

        assert_eq!(response, "Response: 43");
    });

    let subscriber_task = tokio::spawn(async move {
        let mut request = subscriber.receive().await;
        let content = request.take_content().unwrap();
        let response = content + 1;

        request
            .respond(response)
            .await
            .expect("response sent successfully");
    });

    // -- Check
    tokio::try_join!(publisher_task, subscriber_task)?;

    Ok(())
}

#[test_log::test(tokio::test)]
async fn test_request_middleware_with_logging() -> Result<()> {
    // -- Setup & Fixtures
    let mut subscriber = SubscriberImpl::new("subscriber");
    let mut publisher = PublisherImpl::new("publisher", 10);

    subscriber.subscribe_to(&mut publisher)?;

    // -- Exec
    let publisher_task = tokio::spawn(async move {
        // Create a request with logging middleware
        let base_request = RequestImpl::new(100);
        let logging_layer = LoggingRequestLayer::new("test_request");
        
        let logged_request = RequestBuilder::new()
            .layer(logging_layer)
            .request(base_request);

        let response: i32 = publisher
            .request(logged_request)
            .await
            .expect("request published successfully");

        assert_eq!(response, 101);
    });

    let subscriber_task = tokio::spawn(async move {
        let mut request = subscriber.receive().await;
        let content = request.take_content().unwrap();
        let response = content + 1;

        request
            .respond(response)
            .await
            .expect("response sent successfully");
    });

    // -- Check
    tokio::try_join!(publisher_task, subscriber_task)?;

    Ok(())
}

#[test_log::test(tokio::test)]
async fn test_request_middleware_chaining() -> Result<()> {
    // -- Setup & Fixtures
    let mut subscriber = SubscriberImpl::new("subscriber");
    let mut publisher = PublisherImpl::new("publisher", 10);

    subscriber.subscribe_to(&mut publisher)?;

    // -- Exec
    let publisher_task = tokio::spawn(async move {
        // Create a request with both logging and response transformation
        let base_request = RequestImpl::new(10);
        
        let chained_request = RequestBuilder::new()
            .layer(LoggingRequestLayer::new("chained_request"))
            .layer(ResponseTransformRequestLayer::new(|response: i32| {
                format!("Transformed: {}", response * 2)
            }))
            .request(base_request);

        let response: String = publisher
            .request(chained_request)
            .await
            .expect("request published successfully");

        assert_eq!(response, "Transformed: 22");
    });

    let subscriber_task = tokio::spawn(async move {
        let mut request = subscriber.receive().await;
        let content = request.take_content().unwrap();
        let response = content + 1; // 10 + 1 = 11

        request
            .respond(response)
            .await
            .expect("response sent successfully");
    });

    // -- Check
    tokio::try_join!(publisher_task, subscriber_task)?;

    Ok(())
}