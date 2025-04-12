use async_pub_sub::{
    DebuggingPublisherLayer, Publisher, PublisherBuilder, PublisherImpl, SubscriberImpl,
};
use async_pub_sub_macros::{rpc_interface, DerivePublisher, DeriveSubscriber};

use crate::{
    persistency::{PersistencyInterfaceClient, PersistencyInterfaceMessage},
    timer::CacheTimerNotification,
};

const NAME: &str = "Cache";

#[rpc_interface(Debug)]
pub trait CacheInterface {
    async fn get_data(&self) -> String;
    async fn set_data(&mut self, data: String);
}

#[derive(DeriveSubscriber, DerivePublisher)]
pub struct CacheService {
    data: Option<String>,
    #[subscriber(CacheInterfaceMessage)]
    rpc_subscriber: SubscriberImpl<CacheInterfaceMessage>,
    #[publisher(PersistencyInterfaceMessage)]
    persistency_rpc_client: PersistencyInterfaceClient,
    #[subscriber(CacheTimerNotification)]
    timer_notification_subscriber: SubscriberImpl<CacheTimerNotification>,
}

impl CacheService {
    pub fn new() -> Self {
        let persistency_publisher = PublisherBuilder::new()
            .layer(DebuggingPublisherLayer)
            .publisher(PublisherImpl::new(NAME, 10));

        Self {
            data: None,
            rpc_subscriber: SubscriberImpl::new(NAME),
            persistency_rpc_client: PersistencyInterfaceClient::new(persistency_publisher),
            timer_notification_subscriber: SubscriberImpl::new(NAME),
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
