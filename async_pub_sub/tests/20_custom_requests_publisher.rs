use async_pub_sub::{PublisherImpl, Request, Result, SubscriberImpl};

struct CustomRequest<Req, Rsp> {
    content: Req,
    response_callback: Box<dyn FnOnce(Rsp) + Send + Sync>,
}

impl<Req, Rsp> CustomRequest<Req, Rsp> {
    fn new(content: Req, response_callback: impl FnOnce(Rsp) + Send + Sync + 'static) -> Self {
        Self {
            content,
            response_callback: Box::new(response_callback),
        }
    }
}

impl<Req, Rsp> Request for CustomRequest<Req, Rsp> {
    type Content = Req;
    type Response = Rsp;

    fn take_response(
        self,
    ) -> (
        Self,
        futures::future::BoxFuture<'static, Result<Self::Response>>,
    ) {
        todo!()
    }

    fn get_content(&self) -> &Self::Content {
        &self.content
    }

    fn respond(self, response: Self::Response) -> impl Future<Output = Result<()>> {
        async move {
            (self.response_callback)(response);
            Ok(())
        }
    }
}

#[test_log::test(tokio::test)]
async fn test_custom_requests_publisher() -> Result<()> {
    // -- Setup & Fixtures
    let mut subscriber = SubscriberImpl::<CustomRequest<i32, i32>>::new("subscriber");
    let mut publisher = PublisherImpl::new("publisher", 10);

    subscriber.subscribe_to(&mut publisher)?;

    // -- Exec
    let publisher_task = tokio::spawn(async move {
        let (tx, rx) = tokio::sync::oneshot::channel();

        let callback = move |response| {
            let _ = tx.send(response);
        };

        let request = CustomRequest::new(42, callback);
        publisher
            .publish(request)
            .await
            .expect("request published successfully");
        assert_eq!(rx.await.expect("request successul"), 43);
    });

    let subscriber_task = tokio::spawn(async move {
        let request = subscriber.receive().await;
        let response = request.content + 1;

        request
            .respond(response)
            .await
            .expect("response sent successfully");
    });

    // -- Check
    tokio::try_join!(publisher_task, subscriber_task)?;

    Ok(())
}
