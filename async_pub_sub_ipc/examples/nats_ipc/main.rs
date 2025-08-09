use async_pub_sub::{Publisher, PublisherBuilder, Result, Subscriber, SubscriberBuilder};
use async_pub_sub_ipc::{
    NatsPublisher, NatsSubscriber, SerdeJsonDeserializationLayer, SerdeJsonSerializationLayer,
};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Hello,
    World,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let send_first = env::var("SEND_FIRST").unwrap_or_else(|_| "false".to_string()) == "true";
    let app_name = if send_first { "app1" } else { "app2" };
    let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());

    log::info!("Starting {} (send_first: {})", app_name, send_first);

    let message_publisher = PublisherBuilder::new()
        .layer(SerdeJsonSerializationLayer::new())
        .publisher(NatsPublisher::<Message>::new("NatsPublisher", &nats_url).await?);

    let mut message_subscriber = SubscriberBuilder::new()
        .layer(SerdeJsonDeserializationLayer::<Message>::new())
        .subscriber(NatsSubscriber::<Message>::new("NatsSubscriber", &nats_url).await?);

    if send_first {
        log::info!("[{app_name}] Waiting a second before sending initial message");
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        log::info!("[{app_name}] Sending initial message: Hello");
        message_publisher.publish(Message::Hello).await?;
    }

    loop {
        log::info!("[{app_name}] Waiting for messages...");
        let message = message_subscriber.receive().await;
        log::info!("[{app_name}] Received message: {:?}", message);

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let response = match message {
            Message::Hello => Message::World,
            Message::World => Message::Hello,
        };
        log::info!("[{app_name}] Sending response: {:?}", &response);
        message_publisher.publish(response).await?;
    }
}
