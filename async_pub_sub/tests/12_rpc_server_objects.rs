use std::fmt::Display;

use async_pub_sub::{
    Layer, LoggingPublisherLayer, Publisher, PublisherImpl, Request, RequestImpl, Result,
    Subscriber, SubscriberImpl,
};

struct RpcServer<S> {
    subscriber: S,
}

impl<S> RpcServer<S>
where
    S: Subscriber<OutputMessage = Functions>,
{
    pub fn new(subscriber: S) -> Self {
        Self { subscriber }
    }

    pub async fn run(mut self) -> Result<()> {
        loop {
            let request = self.subscriber.receive().await;
            match request {
                Functions::AddOne(mut req) => {
                    let input = req.take_content().unwrap();
                    let response = self.add_one(input).await;
                    req.respond(response).await.unwrap();
                }
                Functions::PrefixWithBar(mut req) => {
                    let input = req.take_content().unwrap();
                    let response = self.prefix_with_bar(input).await;
                    req.respond(response).await.unwrap();
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
    AddOne(RequestImpl<i32, i32>),
    PrefixWithBar(RequestImpl<String, String>),
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
    let mut add_one_request = RequestImpl::new(42);
    let add_one_response = add_one_request.take_response().unwrap();
    publisher
        .publish(Functions::AddOne(add_one_request))
        .await?;
    let add_one_response = add_one_response.await?;

    let mut prefix_with_bar_request = RequestImpl::new("hello".to_string());
    let prefix_with_bar_response = prefix_with_bar_request.take_response().unwrap();
    publisher
        .publish(Functions::PrefixWithBar(prefix_with_bar_request))
        .await?;
    let prefix_with_bar_response = prefix_with_bar_response.await?;

    // -- Check
    assert_eq!(add_one_response, 43);
    assert_eq!(prefix_with_bar_response, "barhello");

    Ok(())
}
