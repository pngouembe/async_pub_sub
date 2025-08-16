use async_pub_sub::{
    Publisher, PublisherBuilder, Request, RequestImpl, Requester, Result, Subscriber,
    SubscriberBuilder,
};
use async_pub_sub_ipc::{
    NatsPublisher, NatsRequestPublisher, NatsRequestSubscriber, NatsSubscriber,
    SerdeJsonDeserializationLayer, SerdeJsonRequestDeserializationLayer,
    SerdeJsonSerializationLayer, SerdeRequestDeserializationLayer, SerdeRequestSerializationLayer,
};
use async_pub_sub_macros::{DeriveSubscriber, rpc_interface};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Hello,
    World,
}

// The RPC interface generates PingContent and PingResponse which are used in rpc_macros_task
// The PingMessage enum contains RequestImpl which can't be serialized, but that's not needed
// for the rpc_macros_task use case
#[rpc_interface(Serialize, Deserialize, Debug)]
trait Ping {
    async fn ping(&self, message: String);
}

#[derive(DeriveSubscriber)]
struct NatsServer<S: Subscriber> {
    subscriber: S,
}

impl<S: Subscriber> NatsServer<S> {
    pub fn new(subscriber: S) -> Self {
        Self { subscriber }
    }
}

impl<S: Subscriber> Ping for NatsServer<S> {
    async fn ping(&self, message: String) {
        log::info!("Pong: {message}")
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let send_first = env::var("SEND_FIRST").unwrap_or_else(|_| "false".to_string()) == "true";
    let app_name = if send_first { "app1" } else { "app2" };
    let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());

    log::info!("Starting {} (send_first: {})", app_name, send_first);

    let _ = tokio::join!(
        pub_sub_task(app_name, &nats_url, send_first),
        rpc_task(app_name, &nats_url, send_first),
        rpc_macros_task(app_name, &nats_url, send_first),
    );

    Ok(())
}

async fn pub_sub_task(app_name: &str, nats_url: &str, send_first: bool) -> Result<()> {
    let message_publisher = PublisherBuilder::new()
        .layer(SerdeJsonSerializationLayer::new())
        .publisher(NatsPublisher::<Message>::new("NatsPublisher", &nats_url).await?);

    let mut message_subscriber = SubscriberBuilder::new()
        .layer(SerdeJsonDeserializationLayer::<Message>::new())
        .subscriber(NatsSubscriber::<Message>::new("NatsSubscriber", &nats_url).await?);

    if send_first {
        log::info!("[{app_name}-pub-sub] Waiting a second before sending initial message");
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        log::info!("[{app_name}-pub-sub] Sending initial message: Hello");
        message_publisher.publish(Message::Hello).await?;
    }

    loop {
        log::info!("[{app_name}-pub-sub] Waiting for messages...");
        let message = message_subscriber.receive().await;
        log::info!("[{app_name}-pub-sub] Received message: {:?}", message);

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let response = match message {
            Message::Hello => Message::World,
            Message::World => Message::Hello,
        };
        log::info!("[{app_name}-pub-sub] Sending response: {:?}", &response);
        message_publisher.publish(response).await?;
    }
}

async fn rpc_task(app_name: &str, nats_url: &str, send_first: bool) -> Result<()> {
    let mut request_counter = 0;

    let request_publisher = PublisherBuilder::new()
        .layer(SerdeRequestSerializationLayer::<String, String>::serde_json())
        .publisher(
            NatsRequestPublisher::<RequestImpl<Bytes, Bytes>>::new(
                "NatsRequestPublisher",
                &nats_url,
            )
            .await?,
        );

    let mut request_subscriber = SubscriberBuilder::new()
        .layer(SerdeJsonRequestDeserializationLayer::<String, String>::new())
        .subscriber(
            NatsSubscriber::<RequestImpl<Bytes, Bytes>>::new("NatsRequestSubscriber", &nats_url)
                .await?,
        );

    if send_first {
        log::info!("[{app_name}-rpc] Waiting a second before sending initial request");
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let request = format!("Hello from {} {}", app_name, request_counter);
            log::info!("[{app_name}-rpc] Sending request: {}", request);

            let request = RequestImpl::new(request);

            let response = request_publisher.request(request).await?;
            request_counter += 1;

            log::info!("[{app_name}-rpc] received response: {}", response);
        }
    } else {
        loop {
            log::info!("[{app_name}-rpc] Waiting for requests...");
            let mut request = request_subscriber.receive().await;
            let content = request.take_content().unwrap();
            log::info!("[{app_name}-rpc] Received request: {}", &content);

            let response = format!("{} Response to: {}", app_name, &content);
            log::info!("[{app_name}-rpc] Sending response: {}", response);
            request.respond(response).await?;
        }
    }
}

async fn rpc_macros_task(app_name: &str, nats_url: &str, send_first: bool) -> Result<()> {
    let mut request_counter = 0;

    if send_first {
        let publisher = PublisherBuilder::new()
            .layer(SerdeRequestSerializationLayer::serde_json())
            .publisher(
                NatsRequestPublisher::<PingMessage>::new("NatsRequestPublisher", nats_url).await?,
            );

        let ping_client = PingClient::new(publisher);

        log::info!("[{app_name}] Waiting for server to start");
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        loop {
            log::info!("[{app_name}] Sending ping with: {}", request_counter);
            ping_client.ping(request_counter.to_string()).await;
            request_counter += 1;
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    } else {
        let subscriber = SubscriberBuilder::new()
            .layer(
                SerdeRequestDeserializationLayer::serde_json(),
            )
            .subscriber(
                NatsRequestSubscriber::<PingMessage>::new("NatsRequestSubscriber", &nats_url)
                    .await?,
            );
        let mut server = NatsServer::new(subscriber);

        server.run().await;
    }

    Ok(())
}
