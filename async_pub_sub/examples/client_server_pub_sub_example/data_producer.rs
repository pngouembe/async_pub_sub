use async_pub_sub::SimpleSubscriber;
use async_pub_sub_macros::{DerivePublisher, DeriveSubscriber};

use crate::{
    cache::{CacheClient, CacheInterface, CacheInterfaceMessage},
    timer::DataProducerTimerNotification,
};

const NAME: &str = "Producer";

#[derive(DerivePublisher, DeriveSubscriber)]
pub struct DataProducerService {
    counter: usize,
    #[publisher(CacheInterfaceMessage)]
    cache_rpc_client: CacheClient,
    #[subscriber(DataProducerTimerNotification)]
    timer_notification_subscriber: SimpleSubscriber<DataProducerTimerNotification>,
}

impl DataProducerService {
    pub fn new() -> Self {
        Self {
            counter: 0,
            cache_rpc_client: CacheClient::new(NAME, 10),
            timer_notification_subscriber: SimpleSubscriber::new(NAME),
        }
    }

    pub async fn run(mut self) {
        log::info!("Starting {}", NAME);
        loop {
            let _ = self.timer_notification_subscriber.receive().await;
            self.counter += 1;

            self.cache_rpc_client
                .set_data(format!("{}: counter = {}", NAME, self.counter))
                .await;
        }
    }
}
