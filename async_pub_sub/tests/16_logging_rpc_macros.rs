use async_pub_sub::{
    DebuggingPublisherLayer, DebuggingSubscriberLayer, PublisherBuilder, PublisherImpl,
    SubscriberBuilder, SubscriberImpl,
};
use async_pub_sub::{Result, Subscriber};

mod interface {
    use async_pub_sub_macros::rpc_interface;
    #[rpc_interface(Debug)]
    pub trait RpcInterface {
        async fn add_one(&self, value: i32) -> i32;
        async fn prefix_with_bar(&self, string: String) -> String;
    }
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

#[test_log::test(tokio::test)]
async fn test_rpc_macros() -> Result<()> {
    let mut rpc_server = server::RpcServer {
        subscriber: SubscriberBuilder::new()
            .layer(DebuggingSubscriberLayer)
            .subscriber(SubscriberImpl::new("rpc_server")),
    };
    let publisher = PublisherBuilder::new()
        .layer(DebuggingPublisherLayer)
        .publisher(PublisherImpl::new("rpc_client", 1));
    let mut rpc_client = interface::RpcInterfaceClient::new(publisher);

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
