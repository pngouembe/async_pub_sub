use server::RpcInterface;
use tokio_pub_sub::{
    LoggingPublisherLayer, PublisherLayer, Result, SimplePublisher, SimpleSubscriber, Subscriber,
};

mod client {
    use tokio_pub_sub::Publisher;
    use tokio_pub_sub_macros::DerivePublisher;

    #[derive(DerivePublisher)]
    pub struct RpcClient<P>
    where
        P: Publisher,
    {
        pub publisher: P,
    }

    mod generated {
        // this should be generated

        use tokio_pub_sub::{Publisher, Request};

        use crate::server::generated::RpcInterfaceFunctions;
        use crate::server::RpcInterface;

        use super::RpcClient;

        // this should be generated with a declarative macro
        impl<P> RpcInterface for RpcClient<P>
        where
            P: Publisher<Message = RpcInterfaceFunctions>,
        {
            async fn add_one(&self, value: i32) -> i32 {
                let (request, response) = Request::new(value);
                self.publisher
                    .publish_event(RpcInterfaceFunctions::AddOne(request))
                    .await
                    .unwrap();
                response.await.unwrap()
            }

            async fn prefix_with_bar(&self, string: String) -> String {
                let (request, response) = Request::new(string);
                self.publisher
                    .publish_event(RpcInterfaceFunctions::PrefixWithBar(request))
                    .await
                    .unwrap();
                response.await.unwrap()
            }
        }
    }
}

mod server {
    use tokio_pub_sub::Subscriber;
    use tokio_pub_sub_macros::DeriveSubscriber;

    pub trait RpcInterface {
        async fn add_one(&self, value: i32) -> i32;
        async fn prefix_with_bar(&self, string: String) -> String;
    }

    #[derive(DeriveSubscriber)]
    pub struct RpcServer<S>
    where
        S: Subscriber,
    {
        pub subscriber: S,
    }

    impl<S> RpcInterface for RpcServer<S>
    where
        S: Subscriber,
    {
        async fn add_one(&self, value: i32) -> i32 {
            value + 1
        }

        async fn prefix_with_bar(&self, string: String) -> String {
            format!("bar{}", string)
        }
    }

    pub mod generated {
        use std::fmt::Display;

        use tokio_pub_sub::{Request, Subscriber};

        use super::RpcInterface;

        // This should be generated by the RpcInterface
        #[derive(Debug)]
        pub enum RpcInterfaceFunctions {
            AddOne(Request<i32, i32>),
            PrefixWithBar(Request<String, String>),
        }

        impl Display for RpcInterfaceFunctions {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }

        // This should be generated by the RpcInterface implementation for RpcServer
        impl<S> super::RpcServer<S>
        where
            S: Subscriber<Message = RpcInterfaceFunctions>,
        {
            pub async fn run(&mut self) {
                loop {
                    let request = self.subscriber.receive().await;

                    match request {
                        RpcInterfaceFunctions::AddOne(req) => {
                            let input = req.content;
                            let response = self.add_one(input).await;
                            req.respond(response);
                        }
                        RpcInterfaceFunctions::PrefixWithBar(req) => {
                            let input = req.content.clone();
                            let response = self.prefix_with_bar(input).await;
                            req.respond(response);
                        }
                    }
                }
            }
        }
    }
}

#[test_log::test(tokio::test)]
async fn test_rpc_macros() -> Result<()> {
    let mut rpc_server = server::RpcServer {
        subscriber: SimpleSubscriber::new("rpc_server"),
    };
    let mut rpc_client = client::RpcClient {
        publisher: LoggingPublisherLayer.layer(SimplePublisher::new("rpc_client", 1)),
    };

    rpc_server.subscribe_to(&mut rpc_client)?;

    tokio::spawn(async move { rpc_server.run().await });

    assert_eq!(rpc_client.add_one(42).await, 43);

    assert_eq!(
        rpc_client.prefix_with_bar("hello".to_string()).await,
        "barhello"
    );

    Ok(())
}
