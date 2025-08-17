use async_pub_sub::{PublisherImpl, Request, RequestImpl, Requester, Result, SubscriberImpl};
use futures::FutureExt;

#[derive(Debug)]
struct CustomContent(i32);

#[derive(Debug, PartialEq)]
struct CustomResponse(i32);

enum RpcRequests {
    AddOne(RequestImpl<i32, i32>),
    Add(RequestImpl<(i32, i32), i32>),
    Custom(RequestImpl<CustomContent, CustomResponse>),
}

#[allow(dead_code)]
enum RpcRequestsContent {
    AddOne(i32),
    Add((i32, i32)),
    Custom(CustomContent),
}

impl From<&mut RpcRequests> for Option<RpcRequestsContent> {
    fn from(value: &mut RpcRequests) -> Self {
        match value {
            RpcRequests::AddOne(req) => req
                .take_content()
                .map(RpcRequestsContent::AddOne),
            RpcRequests::Custom(req) => req
                .take_content()
                .map(RpcRequestsContent::Custom),
            _ => panic!("toto"),
        }
    }
}

#[derive(PartialEq, Debug)]
enum RpcRequestsResponses {
    AddOne(i32),
    Add(i32),
    Custom(CustomResponse),
}

impl Request for RpcRequests {
    type Content = RpcRequestsContent;
    type Response = RpcRequestsResponses;
    type SentResponse = RpcRequestsResponses;

    fn take_response(
        &mut self,
    ) -> Option<futures::future::BoxFuture<'static, Result<Self::Response>>> {
        match self {
            RpcRequests::AddOne(request) => {
                let response_future = request.take_response().unwrap();
                let response_future = async move {
                    let response = response_future.await?;
                    Ok(RpcRequestsResponses::AddOne(response))
                }
                .boxed();
                Some(response_future)
            }
            RpcRequests::Add(request) => {
                let response_future = request.take_response().unwrap();
                let response_future = async move {
                    let response = response_future.await?;
                    Ok(RpcRequestsResponses::Add(response))
                }
                .boxed();
                Some(response_future)
            }
            RpcRequests::Custom(request) => {
                let response_future = request.take_response().unwrap();
                let response_future = async move {
                    let response = response_future.await?;
                    Ok(RpcRequestsResponses::Custom(response))
                }
                .boxed();
                Some(response_future)
            }
        }
    }

    fn take_content(&mut self) -> Option<Self::Content> {
        self.into()
    }

    async fn respond(self, response: Self::SentResponse) -> Result<()> {
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
            RpcRequests::Custom(request) => {
                let RpcRequestsResponses::Custom(response) = response else {
                    panic!("Expected Add response");
                };
                request.respond(response).await
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

        let request = RpcRequests::Custom(RequestImpl::new(CustomContent(2)));
        let response = publisher.request(request).await.expect("titi");
        assert_eq!(response, RpcRequestsResponses::Custom(CustomResponse(4)));
    });

    let subscriber_task = tokio::spawn(async move {
        loop {
            let mut request = subscriber.receive().await;

            let response = match &mut request {
                RpcRequests::AddOne(req) => {
                    let response = req.take_content().unwrap() + 1;
                    RpcRequestsResponses::AddOne(response)
                }
                RpcRequests::Add(req) => {
                    let content = req.take_content().unwrap();
                    let response = content.0 + content.1;
                    RpcRequestsResponses::Add(response)
                }
                RpcRequests::Custom(req) => {
                    let content = req.take_content().unwrap().0;
                    let response = CustomResponse(content + 2);
                    RpcRequestsResponses::Custom(response)
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
