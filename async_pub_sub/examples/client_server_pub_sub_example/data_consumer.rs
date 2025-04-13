use async_pub_sub::{DebuggingPublisherLayer, PublisherBuilder, PublisherImpl, SubscriberImpl};
use async_pub_sub_macros::{DerivePublisher, DeriveSubscriber};

use crate::{
    cache::{CacheInterface, CacheInterfaceClient, CacheInterfaceMessage},
    timer::DataConsumerTimerNotification,
};

const NAME: &str = "Customer";

#[derive(DerivePublisher, DeriveSubscriber)]
pub struct DataConsumerService {
    #[publisher(CacheInterfaceMessage)]
    cache_rpc_client: CacheInterfaceClient,
    #[subscriber(DataConsumerTimerNotification)]
    timer_notification_subscriber: SubscriberImpl<DataConsumerTimerNotification>,
}

impl DataConsumerService {
    pub fn new() -> Self {
        let cache_publisher = PublisherBuilder::new()
            .layer(DebuggingPublisherLayer)
            .publisher(PublisherImpl::new(NAME, 10));

        Self {
            cache_rpc_client: CacheInterfaceClient::new(cache_publisher),
            timer_notification_subscriber: SubscriberImpl::new(NAME),
        }
    }

    pub async fn run(mut self) {
        log::info!("Starting {}", NAME);
        loop {
            let _ = self.timer_notification_subscriber.receive().await;
            let data = self.cache_rpc_client.get_data().await;

            log::info!("[{}] data: {:?}", NAME, data)
        }
    }
}
