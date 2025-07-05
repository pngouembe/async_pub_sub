use std::fmt::Display;

use async_pub_sub::{
    Layer, LoggingPublisherLayer, Publisher, PublisherImpl, Request, Result, Subscriber,
    SubscriberImpl,
};

struct RpcServer<S> {
    subscriber: S,
}

impl<S> RpcServer<S>
where
    S: Subscriber<Message = Functions>,
{
    pub fn new(subscriber: S) -> Self {
        Self { subscriber }
    }

    pub async fn run(mut self) -> Result<()> {
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
        format!("bar{string}")
    }
}

#[derive(Debug)]
enum Functions {
    AddOne(Request<i32, i32>),
    PrefixWithBar(Request<String, String>),
}

impl Display for Functions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[test_log::test(tokio::test)]
async fn test_rpc_server() -> async_pub_sub::Result<()> {
    // -- Setup & Fixtures
    let mut rpc_server = RpcServer::new(SubscriberImpl::new("rpc_server"));
    let mut publisher = LoggingPublisherLayer.layer(PublisherImpl::new("rpc_client", 1));

    rpc_server.subscriber.subscribe_to(&mut publisher)?;

    tokio::spawn(async move { rpc_server.run().await.unwrap() });

    // -- Exec

    let (add_one_request, add_one_response) = Request::new(42);
    publisher
        .publish(Functions::AddOne(add_one_request))
        .await?;
    let add_one_response = add_one_response.await?;

    let (prefix_with_bar_request, prefix_with_bar_response) = Request::new("hello".to_string());
    publisher
        .publish(Functions::PrefixWithBar(prefix_with_bar_request))
        .await?;
    let prefix_with_bar_response = prefix_with_bar_response.await?;

    // -- Check
    assert_eq!(add_one_response, 43);
    assert_eq!(prefix_with_bar_response, "barhello");

    Ok(())
}
