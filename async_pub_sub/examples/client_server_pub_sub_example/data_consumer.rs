use async_pub_sub::SubscriberImpl;
use async_pub_sub_macros::{DerivePublisher, DeriveSubscriber};

use crate::{
    cache::{CacheClient, CacheInterface, CacheInterfaceMessage},
    timer::DataConsumerTimerNotification,
};

const NAME: &str = "Customer";

#[derive(DerivePublisher, DeriveSubscriber)]
pub struct DataConsumerService {
    #[publisher(CacheInterfaceMessage)]
    cache_rpc_client: CacheClient,
    #[subscriber(DataConsumerTimerNotification)]
    timer_notification_subscriber: SubscriberImpl<DataConsumerTimerNotification>,
}

impl DataConsumerService {
    pub fn new() -> Self {
        Self {
            cache_rpc_client: CacheClient::new(NAME, 10),
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
