use async_pub_sub::{
    DebuggingForwarderLayer, DebuggingPublisherLayer, ForwarderBuilder, ForwarderImpl, Publisher,
    PublisherBuilder, PublisherImpl, Subscriber, SubscriberImpl,
};

#[test_log::test(tokio::test)]
async fn test_logging_forwarder() {
    let mut forwarder = ForwarderBuilder::new()
        .layer(DebuggingForwarderLayer)
        .forwarder(ForwarderImpl::new("forwarder"));

    let mut publisher = PublisherBuilder::new()
        .layer(DebuggingPublisherLayer)
        .publisher(PublisherImpl::new("publisher", 1));
    let mut subscriber = SubscriberImpl::new("subscriber");

    forwarder.subscribe_to(&mut publisher).unwrap();
    subscriber.subscribe_to(&mut forwarder).unwrap();

    publisher.publish("Hello, World!").await.unwrap();

    let message = subscriber.receive().await;
    assert_eq!(message, "Hello, World!");
}
