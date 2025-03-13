use std::fmt::Display;

use tokio_pub_sub::{LoggingPublisher, Publisher, Request, Result, SimpleSubscriber, Subscriber};

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
    Foo(Request<Foo, i32>),
    Bar(Request<Bar, String>),
}

impl From<Request<Foo, i32>> for ServiceRequest {
    fn from(request: Request<Foo, i32>) -> Self {
        Self::Foo(request)
    }
}

impl From<Request<Bar, String>> for ServiceRequest {
    fn from(request: Request<Bar, String>) -> Self {
        Self::Bar(request)
    }
}

impl Display for ServiceRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

struct Service {
    subscriber: SimpleSubscriber<ServiceRequest>,
}

impl Service {
    pub fn new() -> Self {
        let subscriber = SimpleSubscriber::new("Service");

        Self { subscriber }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            let request = self.subscriber.receive().await;
            self.handle_request(request);
        }
    }

    fn handle_request(&mut self, request: ServiceRequest) {
        match request {
            ServiceRequest::Foo(request) => {
                let response = self.foo(&request.content);
                request.respond(response);
            }
            ServiceRequest::Bar(request) => {
                let response = self.bar(&request.content);
                request.respond(response);
            }
        }
        {}
    }

    fn foo(&self, value: &Foo) -> i32 {
        let Foo(value) = value;
        value + 1
    }

    fn bar(&mut self, value: &Bar) -> String {
        let Bar(value) = value;
        format!("bar: {}", value)
    }
}

impl Subscriber for Service {
    type Message = ServiceRequest;

    fn get_name(&self) -> &'static str {
        self.subscriber.get_name()
    }

    fn subscribe_to(
        &mut self,
        publisher: &mut impl Publisher<Message = Self::Message>,
    ) -> Result<()> {
        self.subscriber.subscribe_to(publisher)
    }

    fn receive(&mut self) -> impl std::future::Future<Output = ServiceRequest> {
        self.subscriber.receive()
    }
}

#[test_log::test(tokio::test)]
async fn test_direct_rpc() -> Result<()> {
    // -- Setup & Fixtures
    let mut publisher = LoggingPublisher::new("publisher", 1);
    let mut service = Service::new();

    service.subscribe_to(&mut publisher)?;

    tokio::spawn(async move {
        service.run().await.unwrap();
    });

    // -- Exec
    let foo_response = publisher.publish_request(Foo(42)).await?;

    let bar_response = publisher.publish_request(Bar("hello".to_string())).await?;

    // -- Check
    assert_eq!(foo_response, 43);
    assert_eq!(bar_response, "bar: hello");

    Ok(())
}
