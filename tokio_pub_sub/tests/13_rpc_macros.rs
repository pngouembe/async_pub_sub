use tokio_pub_sub::{Result, SimplePublisher, SimpleSubscriber, Subscriber};

mod interface {
    use tokio_pub_sub_macros::rpc_interface;
    #[rpc_interface]
    pub trait RpcInterface {
        async fn add_one(&self, value: i32) -> i32;
        async fn prefix_with_bar(&self, string: String) -> String;
    }
}
mod client {
    use super::interface::{RpcInterfaceClient, RpcInterfaceMessage};
    use tokio_pub_sub::Publisher;
    use tokio_pub_sub_macros::DerivePublisher;

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
    use tokio_pub_sub::Subscriber;
    use tokio_pub_sub_macros::DeriveSubscriber;

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
