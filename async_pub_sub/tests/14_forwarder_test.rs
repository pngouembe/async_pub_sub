use async_pub_sub::{LoggingForwarder, Subscriber};
use tokio_implementations::{publisher::mpsc::MpscPublisher, subscriber::mpsc::MpscSubscriber};

#[test_log::test(tokio::test)]
async fn test_logging_forwarder() {
    let mut forwarder = LoggingForwarder::new("forwarder");
    let mut publisher = MpscPublisher::new("publisher", 1);
    let mut subscriber = MpscSubscriber::new("subscriber");

    forwarder.subscribe_to(&mut publisher).unwrap();
    subscriber.subscribe_to(&mut forwarder).unwrap();

    publisher.publish("Hello, World!").await.unwrap();

    let message = subscriber.receive().await;
    assert_eq!(message, "Hello, World!");
}
