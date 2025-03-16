use std::time::Duration;

use rand::Rng;
use tokio_pub_sub::{Publisher, SimplePublisher};
use tokio_pub_sub_macros::DerivePublisher;

const NAME: &str = "Timer";

// TODO: work on the broadcast

pub struct DataConsumerTimerNotification;
pub struct DataProducerTimerNotification;
pub struct CacheTimerNotification;

#[derive(DerivePublisher)]
pub struct TimerService {
    #[publisher(DataConsumerTimerNotification)]
    data_consumer_publisher: SimplePublisher<DataConsumerTimerNotification>,
    #[publisher(DataProducerTimerNotification)]
    data_producer_publisher: SimplePublisher<DataProducerTimerNotification>,
    #[publisher(CacheTimerNotification)]
    cache_publisher: SimplePublisher<CacheTimerNotification>,
}

impl TimerService {
    pub fn new() -> Self {
        Self {
            data_consumer_publisher: SimplePublisher::new(NAME, 10),
            data_producer_publisher: SimplePublisher::new(NAME, 10),
            cache_publisher: SimplePublisher::new(NAME, 10),
        }
    }

    pub async fn run(self) {
        let data_consumer_task = async move {
            let mut rng = rand::rng();
            loop {
                let second_count = rng.random_range(3..6);
                log::info!("[{}] notifying data customer in {}s", NAME, second_count);
                tokio::time::sleep(Duration::from_secs(second_count)).await;
                self.data_consumer_publisher
                    .publish_event(DataConsumerTimerNotification)
                    .await
                    .unwrap();
            }
        };

        let data_producer_task = async move {
            let mut rng = rand::rng();
            loop {
                let second_count = rng.random_range(1..4);
                log::info!("[{}] notifying data producer in {}s", NAME, second_count);
                tokio::time::sleep(Duration::from_secs(second_count)).await;
                self.data_producer_publisher
                    .publish_event(DataProducerTimerNotification)
                    .await
                    .unwrap();
            }
        };

        let cache_publisher = async move {
            let mut rng = rand::rng();
            loop {
                let second_count = rng.random_range(5..8);
                log::info!("[{}] notifying data cache in {}s", NAME, second_count);
                tokio::time::sleep(Duration::from_secs(second_count)).await;
                self.cache_publisher
                    .publish_event(CacheTimerNotification)
                    .await
                    .unwrap();
            }
        };

        tokio::join!(data_consumer_task, data_producer_task, cache_publisher);
    }
}
