use server::RpcInterface;
use tokio_pub_sub::{Result, SimplePublisher, SimpleSubscriber, Subscriber};

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

        use crate::server::RpcInterface;
        use crate::server::RpcInterfaceFunctions;

        use super::RpcClient;

        // this should be generated with a declarative macro
        impl<P> RpcInterface for RpcClient<P>
        where
            P: Publisher<Message = RpcInterfaceFunctions>,
        {
            type RpcMessage = P::Message;

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
    use tokio_pub_sub_macros::{rpc_interface, DeriveSubscriber};

    pub trait RpcInterface {
        type RpcMessage: Send + 'static;

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

    #[rpc_interface]
    impl<S> RpcInterface for RpcServer<S>
    where
        S: Subscriber,
    {
        type RpcMessage = <S as Subscriber>::Message;

        async fn add_one(&self, value: i32) -> i32 {
            value + 1
        }

        async fn prefix_with_bar(&self, string: String) -> String {
            format!("bar{}", string)
        }
    }
}

#[test_log::test(tokio::test)]
async fn test_rpc_macros() -> Result<()> {
    let mut rpc_server = server::RpcServer {
        subscriber: SimpleSubscriber::new("rpc_server"),
    };
    let mut rpc_client = client::RpcClient {
        publisher: SimplePublisher::new("rpc_client", 1),
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
