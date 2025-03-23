mod cache;
mod data_consumer;
mod data_producer;
mod persistency;
mod timer;

use cache::CacheService;
use data_consumer::DataConsumerService;
use data_producer::DataProducerService;
use persistency::PersistencyService;
use timer::{CacheTimerNotification, TimerService};
use async_pub_sub::Result;
use async_pub_sub_macros::routes;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder().format_target(false).init();

    log::info!("Starting Main");
    let mut timer_service = TimerService::new();
    let mut cache_service = CacheService::new();
    let mut persistency_service = PersistencyService::new();
    let mut data_producer_service = DataProducerService::new();
    let mut data_consumer_service = DataConsumerService::new();

    routes!(
        data_consumer_service -> cache_service,
        data_producer_service -> cache_service,
        timer_service -> cache_service: CacheTimerNotification,
        cache_service -> persistency_service,
        timer_service -> data_consumer_service,
        timer_service -> data_producer_service,
    )?;

    tokio::join!(
        cache_service.run(),
        data_consumer_service.run(),
        data_producer_service.run(),
        persistency_service.run(),
        timer_service.run(),
    );

    Ok(())
}
