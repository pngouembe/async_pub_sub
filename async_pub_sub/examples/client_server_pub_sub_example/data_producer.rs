use async_pub_sub::SubscriberImpl;
use async_pub_sub_macros::{DerivePublisher, DeriveSubscriber};

use crate::{
    cache::{CacheInterface, CacheInterfaceClient, CacheInterfaceMessage},
    timer::DataProducerTimerNotification,
};

const NAME: &str = "Producer";

#[derive(DerivePublisher, DeriveSubscriber)]
pub struct DataProducerService {
    counter: usize,
    #[publisher(CacheInterfaceMessage)]
    cache_rpc_client: CacheInterfaceClient,
    #[subscriber(DataProducerTimerNotification)]
    timer_notification_subscriber: SubscriberImpl<DataProducerTimerNotification>,
}

impl DataProducerService {
    pub fn new() -> Self {
        let cache_publisher = async_pub_sub::PublisherBuilder::new()
            .layer(async_pub_sub::DebuggingPublisherLayer)
            .publisher(async_pub_sub::PublisherImpl::new(NAME, 10));

        Self {
            counter: 0,
            cache_rpc_client: CacheInterfaceClient::new(cache_publisher),
            timer_notification_subscriber: SubscriberImpl::new(NAME),
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
