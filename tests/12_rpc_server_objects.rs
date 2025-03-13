use std::fmt::Display;

use tokio_pub_sub::{
    LoggingPublisherLayer, Publisher, PublisherLayer, Request, Result, SimplePublisher,
    SimpleSubscriber, Subscriber,
};

struct RpcServer<S> {
    subscriber: S,
}

impl<S> RpcServer<S> {
    pub fn new(subscriber: S) -> Self
    where
        S: Subscriber<Functions>,
    {
        Self { subscriber }
    }

    pub async fn run(mut self) -> Result<()>
    where
        S: Subscriber<Functions>,
    {
        loop {
            let request = self.subscriber.receive().await;
            match request {
                Functions::AddOne(req) => {
                    let input = req.content;
                    let response = self.add_one(input).await;
                    req.respond(response);
                }
                Functions::PrefixWithBar(req) => {
                    let input = req.content.clone();
                    let response = self.prefix_with_bar(input).await;
                    req.respond(response);
                }
            }
        }
    }

    pub async fn add_one(&self, value: i32) -> i32 {
        value + 1
    }

    pub async fn prefix_with_bar(&self, string: String) -> String {
        format!("bar{}", string)
    }
}

#[derive(Debug)]
enum Functions {
    AddOne(Request<i32, i32>),
    PrefixWithBar(Request<String, String>),
}

impl Display for Functions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[test_log::test(tokio::test)]
async fn test_rpc_server() -> tokio_pub_sub::Result<()> {
    // -- Setup & Fixtures
    let mut rpc_server = RpcServer::new(SimpleSubscriber::new("rpc_server"));
    let mut publisher = LoggingPublisherLayer.layer(SimplePublisher::new("rpc_client", 1));

    rpc_server.subscriber.subscribe_to(&mut publisher)?;

    tokio::spawn(async move { rpc_server.run().await.unwrap() });

    // -- Exec

    let (add_one_request, add_one_response) = Request::new(42);
    publisher
        .publish_event(Functions::AddOne(add_one_request))
        .await?;
    let add_one_response = add_one_response.await?;

    let (prefix_with_bar_request, prefix_with_bar_response) = Request::new("hello".to_string());
    publisher
        .publish_event(Functions::PrefixWithBar(prefix_with_bar_request))
        .await?;
    let prefix_with_bar_response = prefix_with_bar_response.await?;

    // -- Check
    assert_eq!(add_one_response, 43);
    assert_eq!(prefix_with_bar_response, "barhello");

    Ok(())
}
