use async_pub_sub::{PublisherImpl, Request, RequestImpl, Requester, Result, SubscriberImpl};
use futures::FutureExt;

enum RpcRequests {
    AddOne(RequestImpl<i32, i32>),
    Add(RequestImpl<(i32, i32), i32>),
}

#[derive(PartialEq, Debug)]
enum RpcRequestsResponses {
    AddOne(i32),
    Add(i32),
}

impl Request for RpcRequests {
    type Content = RpcRequests;
    type Response = RpcRequestsResponses;

    fn take_response(
        self,
    ) -> (
        Self,
        futures::future::BoxFuture<'static, Result<Self::Response>>,
    ) {
        match self {
            RpcRequests::AddOne(request) => {
                let (request, response_future) = request.take_response();
                let response_future = async move {
                    let response = response_future.await?;
                    Ok(RpcRequestsResponses::AddOne(response))
                }
                .boxed();
                (RpcRequests::AddOne(request), response_future)
            }
            RpcRequests::Add(request) => {
                let (request, response_future) = request.take_response();
                let response_future = async move {
                    let response = response_future.await?;
                    Ok(RpcRequestsResponses::Add(response))
                }
                .boxed();
                (RpcRequests::Add(request), response_future)
            }
        }
    }

    fn get_content(&self) -> &Self::Content {
        unimplemented!()
    }

    fn respond(self, response: Self::Response) -> impl Future<Output = Result<()>> {
        async move {
            match self {
                RpcRequests::AddOne(request) => {
                    let RpcRequestsResponses::AddOne(response) = response else {
                        panic!("Expected AddOne response");
                    };
                    request.respond(response).await
                }
                RpcRequests::Add(request) => {
                    let RpcRequestsResponses::Add(response) = response else {
                        panic!("Expected Add response");
                    };
                    request.respond(response).await
                }
            }
        }
    }
}

#[test_log::test(tokio::test)]
async fn test_request_with_enum() -> Result<()> {
    // -- Setup & Fixtures
    let mut subscriber = SubscriberImpl::<RpcRequests>::new("subscriber");
    let mut publisher = PublisherImpl::new("publisher", 10);

    subscriber.subscribe_to(&mut publisher)?;

    // -- Exec
    let publisher_task = tokio::spawn(async move {
        let request = RpcRequests::AddOne(RequestImpl::new(42));
        let response = publisher.request(request).await.expect("toto");

        assert_eq!(response, RpcRequestsResponses::AddOne(43));

        let request = RpcRequests::Add(RequestImpl::new((1, 2)));
        let response = publisher.request(request).await.expect("tata");
        assert_eq!(response, RpcRequestsResponses::Add(3));
    });

    let subscriber_task = tokio::spawn(async move {
        loop {
            // Receive requests until we get the expected one
            let request = subscriber.receive().await;

            let response = match &request {
                RpcRequests::AddOne(req) => {
                    let response = req.content + 1;
                    RpcRequestsResponses::AddOne(response)
                }
                RpcRequests::Add(req) => {
                    let response = req.content.0 + req.content.1;
                    RpcRequestsResponses::Add(response)
                }
            };

            request
                .respond(response)
                .await
                .expect("response sent successfully");
        }
    });

    // -- Check
    tokio::select! {
        _ = publisher_task => {},
        _ = subscriber_task => {},
    }

    Ok(())
}
