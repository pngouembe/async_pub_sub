use async_pub_sub::{DebugingPublisherLayer, PublisherBuilder};
use async_pub_sub::{Result, Subscriber};

mod interface {
    use async_pub_sub_macros::rpc_interface;
    #[rpc_interface]
    pub trait RpcInterface {
        async fn add_one(&self, value: i32) -> i32;
        async fn prefix_with_bar(&self, string: String) -> String;
    }
}
mod client {
    use super::interface::{RpcInterfaceClient, RpcInterfaceMessage};
    use async_pub_sub::Publisher;
    use async_pub_sub_macros::DerivePublisher;

    #[derive(DerivePublisher)]
    pub struct RpcClient<P>
    where
        P: Publisher,
    {
        pub publisher: P,
    }

    impl<P> RpcInterfaceClient for RpcClient<P> where P: Publisher<Message = RpcInterfaceMessage> {}
}

mod server {
    use super::RpcInterface;
    use async_pub_sub::Subscriber;
    use async_pub_sub_macros::DeriveSubscriber;

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
}

use interface::RpcInterface;
use interface::RpcInterfaceServer;
use tokio_implementations::publisher::mpsc::MpscPublisher;
use tokio_implementations::subscriber::mpsc::MpscSubscriber;

#[test_log::test(tokio::test)]
async fn test_rpc_macros() -> Result<()> {
    let mut rpc_server = server::RpcServer {
        subscriber: MpscSubscriber::new("rpc_server"),
    };
    let mut rpc_client = client::RpcClient {
        publisher: PublisherBuilder::new(MpscPublisher::new("rpc_client", 1))
            .with_layer(DebugingPublisherLayer)
            .build(),
    };

    rpc_server.subscribe_to(&mut rpc_client)?;

    let future = async move { rpc_server.run().await };

    tokio::spawn(future);

    assert_eq!(rpc_client.add_one(42).await, 43);

    assert_eq!(
        rpc_client.prefix_with_bar("hello".to_string()).await,
        "barhello"
    );

    Ok(())
}
