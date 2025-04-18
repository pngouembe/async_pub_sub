use async_pub_sub::{PublisherImpl, PublisherWrapper, Result, SubscriberImpl};

// #[derive(DerivePublisher)]
struct MultiPub {
    // #[publisher(i32)]
    publisher_a: PublisherImpl<i32>,
    // #[publisher(String)]
    publisher_b: PublisherImpl<String>,
}

impl PublisherWrapper<i32> for MultiPub {
    fn get_publisher(&self) -> &dyn async_pub_sub::Publisher<Message = i32> {
        &self.publisher_a
    }

    fn get_publisher_mut(&mut self) -> &mut dyn async_pub_sub::Publisher<Message = i32> {
        &mut self.publisher_a
    }
}

impl PublisherWrapper<String> for MultiPub {
    fn get_publisher(&self) -> &dyn async_pub_sub::Publisher<Message = String> {
        &self.publisher_b
    }

    fn get_publisher_mut(&mut self) -> &mut dyn async_pub_sub::Publisher<Message = String> {
        &mut self.publisher_b
    }
}

impl MultiPub {
    fn new() -> Self {
        Self {
            publisher_a: PublisherImpl::new("publisher", 1),
            publisher_b: PublisherImpl::new("publisher", 1),
        }
    }
}

#[tokio::test]
async fn test_multi_pub() -> Result<()> {
    let mut subscriber1 = SubscriberImpl::<i32>::new("subscriber1");
    let mut subscriber2 = SubscriberImpl::<String>::new("subscriber2");

    let mut publisher = MultiPub::new();
    subscriber1.subscribe_to(PublisherWrapper::<i32>::get_publisher_mut(&mut publisher))?;
    subscriber2.subscribe_to(PublisherWrapper::<String>::get_publisher_mut(
        &mut publisher,
    ))?;

    publisher.publish(42).await?;
    let message = subscriber1.receive().await;
    assert_eq!(message, 42);

    publisher.publish("toto".to_string()).await?;
    let message = subscriber2.receive().await;
    assert_eq!(message, "toto");

    Ok(())
}
