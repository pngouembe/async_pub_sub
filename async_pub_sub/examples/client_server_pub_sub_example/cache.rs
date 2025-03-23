use async_pub_sub::{
    DebugingPublisherLayer, Publisher, PublisherBuilder, SimplePublisher, SimpleSubscriber,
};
use async_pub_sub_macros::{rpc_interface, DerivePublisher, DeriveSubscriber};

use crate::{
    persistency::{PersistencyClient, PersistencyInterface, PersistencyInterfaceMessage},
    timer::CacheTimerNotification,
};

const NAME: &str = "Cache";

#[rpc_interface]
pub trait CacheInterface {
    async fn get_data(&self) -> String;
    async fn set_data(&mut self, data: String);
}

#[derive(DeriveSubscriber, DerivePublisher)]
pub struct CacheService {
    data: Option<String>,
    #[subscriber(CacheInterfaceMessage)]
    rpc_subscriber: SimpleSubscriber<CacheInterfaceMessage>,
    #[publisher(PersistencyInterfaceMessage)]
    persistency_rpc_client: PersistencyClient,
    #[subscriber(CacheTimerNotification)]
    timer_notification_subscriber: SimpleSubscriber<CacheTimerNotification>,
}

impl CacheService {
    pub fn new() -> Self {
        Self {
            data: None,
            rpc_subscriber: SimpleSubscriber::new(NAME),
            persistency_rpc_client: PersistencyClient::new(NAME, 10),
            timer_notification_subscriber: SimpleSubscriber::new(NAME),
        }
    }

    pub async fn run(mut self) {
        log::info!("Starting {}", NAME);
        loop {
            tokio::select! {
                request = self.rpc_subscriber.receive() => {
                    CacheInterfaceServer::handle_request(&mut self, request).await
                }
                _ = self.timer_notification_subscriber.receive() => {
                    log::info!("[{}] store data in persistency", NAME);
                    self.persistency_rpc_client.store_data(self.data.clone().unwrap_or_default().into_bytes()).await
                }
            }
        }
    }
}

impl CacheInterface for CacheService {
    async fn get_data(&self) -> String {
        self.data.clone().unwrap_or_default()
    }

    async fn set_data(&mut self, data: String) {
        self.data = Some(data);
    }
}

#[derive(DerivePublisher)]
pub struct CacheClient {
    #[publisher(CacheInterfaceMessage)]
    rpc_publisher: Box<dyn Publisher<Message = CacheInterfaceMessage>>,
}

impl CacheClient {
    pub fn new(name: &'static str, buffer_size: usize) -> Self {
        Self {
            rpc_publisher: Box::new(
                PublisherBuilder::new(SimplePublisher::new(name, buffer_size))
                    .with_layer(DebugingPublisherLayer)
                    .build(),
            ),
        }
    }
}

impl CacheInterfaceClient for CacheClient {}
