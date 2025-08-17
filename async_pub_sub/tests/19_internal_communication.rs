use async_pub_sub::{Publisher, PublisherImpl, SubscriberImpl, macros::routes};

struct MyService {
    publisher1: Box<dyn Publisher<InputMessage = String, OutputMessage = String> + Send + Sync>,
    publisher2: Box<dyn Publisher<InputMessage = String, OutputMessage = String> + Send + Sync>,
    // TODO: find a way to make the subscriber trait dyn compatible
    subscriber: SubscriberImpl<String>,
}

impl MyService {
    fn new() -> Self {
        let publisher1 = Box::new(PublisherImpl::<String>::new("my_service", 1));
        let publisher2 = Box::new(PublisherImpl::<String>::new("my_service", 1));
        let subscriber = SubscriberImpl::<String>::new("my_service");
        Self {
            publisher1,
            publisher2,
            subscriber,
        }
    }

    fn init(&mut self) {
        routes! { self.publisher1 -> self.subscriber, self.publisher2 -> self.subscriber: String }
            .unwrap();
    }
}

#[tokio::test]
async fn test_internal_communication() {
    let mut service = MyService::new();
    service.init();

    // Simulate publishing messages
    service
        .publisher1
        .publish("Hello from publisher1".to_string())
        .await
        .unwrap();
    service
        .publisher2
        .publish("Hello from publisher2".to_string())
        .await
        .unwrap();

    // Simulate receiving messages
    let message1 = service.subscriber.receive().await;
    let message2 = service.subscriber.receive().await;

    assert_eq!(message1, "Hello from publisher1");
    assert_eq!(message2, "Hello from publisher2");
}
