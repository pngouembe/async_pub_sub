use tokio_pub_sub::{MultiPublisher, Publisher, Result, SimplePublisher, SimpleSubscriber};

// TODO: update DerivePublisher macro to support this use case
// TODO: update DerivePublisher macro to allow to indicate concrete publisher with a macro attribute

struct MultiPub {
    publisher_a: SimplePublisher<i32>,
    publisher_b: SimplePublisher<String>,
}

impl MultiPub {
    fn new() -> Self {
        Self {
            publisher_a: SimplePublisher::new("publisher", 1),
            publisher_b: SimplePublisher::new("publisher", 1),
        }
    }
}

impl MultiPublisher<i32> for MultiPub {
    fn get_publisher(&self) -> &impl Publisher<Message = i32> {
        &self.publisher_a
    }

    fn get_publisher_mut(&mut self) -> &mut impl Publisher<Message = i32> {
        &mut self.publisher_a
    }
}

impl MultiPublisher<String> for MultiPub {
    fn get_publisher(&self) -> &impl Publisher<Message = String> {
        &self.publisher_b
    }

    fn get_publisher_mut(&mut self) -> &mut impl Publisher<Message = String> {
        &mut self.publisher_b
    }
}

#[tokio::test]
async fn test_multi_pub() -> Result<()> {
    let mut subscriber1 = SimpleSubscriber::<i32>::new("subscriber1");
    let mut subscriber2 = SimpleSubscriber::<String>::new("subscriber2");

    let mut publisher = MultiPub::new();
    subscriber1.subscribe_to(&mut publisher)?;
    subscriber2.subscribe_to(&mut publisher)?;

    Ok(())
}
