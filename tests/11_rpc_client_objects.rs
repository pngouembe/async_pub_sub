use std::fmt::Display;

use tokio_pub_sub::{
    LoggingPublisherLayer, Publisher, PublisherLayer, Request, Result, SimplePublisher,
    SimpleSubscriber,
};

struct RpcClient<P> {
    publisher: P,
}

impl<P> RpcClient<P> {
    pub fn new(publisher: P) -> Self
    where
        P: Publisher<Functions>,
    {
        Self { publisher }
    }

    pub async fn add_one(&self, value: i32) -> Result<i32>
    where
        P: Publisher<Functions>,
    {
        let (request, response) = Request::<i32, i32>::new(value);
        self.publisher
            .publish_event(Functions::AddOne(request))
            .await?;
        let response = response.await?;
        Ok(response)
    }

    pub async fn prefix_with_bar(&self, string: String) -> Result<String>
    where
        P: Publisher<Functions>,
    {
        let (request, response) = Request::<String, String>::new(string);
        self.publisher
            .publish_event(Functions::PrefixWithBar(request))
            .await?;
        let response = response.await?;
        Ok(response)
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
async fn test_rpc_client_layer() -> tokio_pub_sub::Result<()> {
    // -- Setup & Fixtures
    let mut subscriber = SimpleSubscriber::new("rpc_server");
    let mut rpc_client =
        RpcClient::new(LoggingPublisherLayer.layer(SimplePublisher::new("rpc_client", 1)));

    subscriber.subscribe_to(&mut rpc_client.publisher)?;

    tokio::spawn(async move {
        loop {
            match subscriber.receive().await {
                Functions::AddOne(req) => {
                    let response = req.content + 1;
                    req.respond(response)
                }
                Functions::PrefixWithBar(req) => {
                    let response = format!("bar{}", req.content);
                    req.respond(response);
                }
            }
        }
    });

    // -- Exec

    let add_one_response = rpc_client.add_one(42).await?;
    let prefix_with_bar_response = rpc_client.prefix_with_bar("hello".to_string()).await?;

    // -- Check
    assert_eq!(add_one_response, 43);
    assert_eq!(prefix_with_bar_response, "barhello");

    Ok(())
}
