use async_pub_sub::{PublisherImpl, SubscriberImpl};
use async_pub_sub_macros::{rpc_interface, DerivePublisher, DeriveSubscriber};

const NAME: &str = "Persistency";

// TODO: find a way to handle references or forbid their use
#[rpc_interface]
pub trait PersistencyInterface {
    async fn get_data(&self) -> Vec<u8>;
    async fn store_data(&mut self, data: Vec<u8>);
}

#[derive(DeriveSubscriber)]
pub struct PersistencyService {
    data: Vec<u8>,
    #[subscriber(PersistencyInterfaceMessage)]
    rpc_subscriber: SubscriberImpl<PersistencyInterfaceMessage>,
}

impl PersistencyService {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            rpc_subscriber: SubscriberImpl::new(NAME),
        }
    }

    pub async fn run(mut self) {
        log::info!("Starting {}", NAME);
        PersistencyInterfaceServer::run(&mut self).await
    }
}

impl PersistencyInterface for PersistencyService {
    async fn get_data(&self) -> Vec<u8> {
        log::info!("[{}] getting data from persistency", NAME);
        self.data.clone()
    }

    async fn store_data(&mut self, data: Vec<u8>) {
        log::info!("[{}] storing data in persistency", NAME);
        self.data = data
    }
}

#[derive(DerivePublisher)]
pub struct PersistencyClient {
    #[publisher(PersistencyInterfaceMessage)]
    rpc_publisher: PublisherImpl<PersistencyInterfaceMessage>,
}

impl PersistencyClient {
    pub fn new(name: &'static str, buffer_size: usize) -> Self {
        Self {
            rpc_publisher: PublisherImpl::new(name, buffer_size),
        }
    }
}

impl PersistencyInterfaceClient for PersistencyClient {}
