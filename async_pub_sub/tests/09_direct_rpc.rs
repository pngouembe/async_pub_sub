use std::fmt::Display;

use async_pub_sub::{
    PublisherImpl, PublisherWrapper, Request, RequestImpl, Result, Subscriber, SubscriberImpl,
};
use futures::{FutureExt, future::BoxFuture};

#[derive(Debug, PartialEq)]
struct Foo(i32);

impl Display for Foo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(Debug, PartialEq)]
struct Bar(String);

impl Display for Bar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(Debug)]
enum ServiceRequest {
    Foo(RequestImpl<Foo, i32>),
    Bar(RequestImpl<Bar, String>),
}

impl From<RequestImpl<Foo, i32>> for ServiceRequest {
    fn from(request: RequestImpl<Foo, i32>) -> Self {
        Self::Foo(request)
    }
}

impl From<RequestImpl<Bar, String>> for ServiceRequest {
    fn from(request: RequestImpl<Bar, String>) -> Self {
        Self::Bar(request)
    }
}

impl Display for ServiceRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

struct Service {
    subscriber: SubscriberImpl<ServiceRequest>,
}

impl Service {
    pub fn new() -> Self {
        let subscriber = SubscriberImpl::new("Service");

        Self { subscriber }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            let request = self.subscriber.receive().await;
            self.handle_request(request).await;
        }
    }

    async fn handle_request(&mut self, request: ServiceRequest) {
        match request {
            ServiceRequest::Foo(mut request) => {
                let content = request.take_content().unwrap();
                let response = self.foo(&content);
                request.respond(response).await.unwrap();
            }
            ServiceRequest::Bar(mut request) => {
                let content = request.take_content().unwrap();
                let response = self.bar(&content);
                request.respond(response).await.unwrap();
            }
        }
    }

    fn foo(&self, value: &Foo) -> i32 {
        let Foo(value) = value;
        value + 1
    }

    fn bar(&mut self, value: &Bar) -> String {
        let Bar(value) = value;
        format!("bar: {value}")
    }
}

impl Subscriber for Service {
    type InputMessage = ServiceRequest;
    type OutputMessage = ServiceRequest;

    fn get_name(&self) -> &'static str {
        self.subscriber.get_name()
    }

    fn subscribe_to<P, Input>(&mut self, publisher: &mut P) -> Result<()>
    where
        P: PublisherWrapper<Input, Self::InputMessage>,
        Input: Send + 'static,
    {
        self.subscriber.subscribe_to(publisher)
    }

    fn receive(&mut self) -> BoxFuture<'_, Self::InputMessage> {
        self.subscriber.receive().boxed()
    }
}

#[test_log::test(tokio::test)]
async fn test_direct_rpc() -> Result<()> {
    // -- Setup & Fixtures
    let mut publisher = PublisherImpl::new("publisher", 1);
    let mut service = Service::new();

    service.subscribe_to(&mut publisher)?;

    tokio::spawn(async move {
        service.run().await.unwrap();
    });

    // -- Exec & Check
    let mut request = RequestImpl::new(Foo(42));
    let response = request.take_response().unwrap();
    publisher
        .publish(request.into())
        .await
        .expect("request published successfully");
    assert_eq!(response.await.expect("request successul"), 43);

    let mut request = RequestImpl::new(Bar("hello".to_string()));
    let response = request.take_response().unwrap();
    publisher
        .publish(request.into())
        .await
        .expect("request published successfully");
    assert_eq!(response.await.expect("request successul"), "bar: hello");

    Ok(())
}
