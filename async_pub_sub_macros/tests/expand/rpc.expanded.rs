#![allow(unused_imports)]
use async_pub_sub_macros::rpc_interface;
#[allow(async_fn_in_trait)]
pub trait RpcInterface {
    async fn add_one(&self, value: i32) -> i32;
    async fn add(&self, left: i32, right: i32) -> i32;
    async fn prefix_with_bar(&self, string: String) -> String;
    async fn get_toto(&self) -> String;
    async fn set_tata(&mut self, tata: String);
}
pub enum RpcInterfaceMessage {
    AddOne(async_pub_sub::Request<i32, i32>),
    Add(async_pub_sub::Request<(i32, i32), i32>),
    PrefixWithBar(async_pub_sub::Request<String, String>),
    GetToto(async_pub_sub::Request<(), String>),
    SetTata(async_pub_sub::Request<String, ()>),
}
pub struct RpcInterfaceClient {
    #[publisher(RpcInterfaceMessage)]
    pub publisher: Box<
        dyn async_pub_sub::Publisher<Message = RpcInterfaceMessage> + Send,
    >,
}
impl async_pub_sub::Publisher for RpcInterfaceClient {
    type Message = RpcInterfaceMessage;
    fn get_name(&self) -> &'static str {
        async_pub_sub::Publisher::get_name(&self.publisher)
    }
    fn publish(
        &self,
        message: Self::Message,
    ) -> async_pub_sub::futures::future::BoxFuture<async_pub_sub::Result<()>> {
        async_pub_sub::Publisher::publish(&self.publisher, message)
    }
    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> async_pub_sub::Result<
        std::pin::Pin<
            Box<
                dyn async_pub_sub::futures::Stream<
                    Item = Self::Message,
                > + Send + Sync + 'static,
            >,
        >,
    > {
        async_pub_sub::Publisher::get_message_stream(
            &mut self.publisher,
            subscriber_name,
        )
    }
}
impl RpcInterfaceClient {
    pub fn new<P>(publisher: P) -> Self
    where
        P: async_pub_sub::Publisher<Message = RpcInterfaceMessage> + Send + 'static,
    {
        Self {
            publisher: Box::new(publisher),
        }
    }
}
impl RpcInterface for RpcInterfaceClient {
    fn add_one(&self, value: i32) -> async_pub_sub::futures::future::BoxFuture<i32> {
        let (request, response) = async_pub_sub::Request::new(value);
        let publish_future = self
            .publisher
            .publish(RpcInterfaceMessage::AddOne(request));
        {
            use async_pub_sub::futures::FutureExt;
            async move {
                publish_future.await.expect("failed to publish add_one request");
                response.await.expect("failed to receive add_one response")
            }
                .boxed()
        }
    }
    fn add(
        &self,
        left: i32,
        right: i32,
    ) -> async_pub_sub::futures::future::BoxFuture<i32> {
        let (request, response) = async_pub_sub::Request::new((left, right));
        let publish_future = self.publisher.publish(RpcInterfaceMessage::Add(request));
        {
            use async_pub_sub::futures::FutureExt;
            async move {
                publish_future.await.expect("failed to publish add request");
                response.await.expect("failed to receive add response")
            }
                .boxed()
        }
    }
    fn prefix_with_bar(
        &self,
        string: String,
    ) -> async_pub_sub::futures::future::BoxFuture<String> {
        let (request, response) = async_pub_sub::Request::new(string);
        let publish_future = self
            .publisher
            .publish(RpcInterfaceMessage::PrefixWithBar(request));
        {
            use async_pub_sub::futures::FutureExt;
            async move {
                publish_future.await.expect("failed to publish prefix_with_bar request");
                response.await.expect("failed to receive prefix_with_bar response")
            }
                .boxed()
        }
    }
    fn get_toto(&self) -> async_pub_sub::futures::future::BoxFuture<String> {
        let (request, response) = async_pub_sub::Request::new(());
        let publish_future = self
            .publisher
            .publish(RpcInterfaceMessage::GetToto(request));
        {
            use async_pub_sub::futures::FutureExt;
            async move {
                publish_future.await.expect("failed to publish get_toto request");
                response.await.expect("failed to receive get_toto response")
            }
                .boxed()
        }
    }
    fn set_tata(
        &mut self,
        tata: String,
    ) -> async_pub_sub::futures::future::BoxFuture<()> {
        let (request, response) = async_pub_sub::Request::new(tata);
        let publish_future = self
            .publisher
            .publish(RpcInterfaceMessage::SetTata(request));
        {
            use async_pub_sub::futures::FutureExt;
            async move {
                publish_future.await.expect("failed to publish set_tata request");
                response.await.expect("failed to receive set_tata response")
            }
                .boxed()
        }
    }
}
pub trait RpcInterfaceServer: async_pub_sub::SubscriberWrapper<
        RpcInterfaceMessage,
    > + RpcInterface {
    async fn run(&mut self) {
        loop {
            let request = self.receive().await;
            self.handle_request(request).await;
        }
    }
    async fn handle_request(&mut self, request: RpcInterfaceMessage) {
        match request {
            RpcInterfaceMessage::AddOne(req) => {
                let async_pub_sub::Request { content, response_sender } = req;
                let response = <Self as RpcInterface>::add_one(self, content).await;
                response_sender.send(response).expect("failed to send response");
            }
            RpcInterfaceMessage::Add(req) => {
                let async_pub_sub::Request { content, response_sender } = req;
                let (left, right) = content;
                let response = <Self as RpcInterface>::add(self, left, right).await;
                response_sender.send(response).expect("failed to send response");
            }
            RpcInterfaceMessage::PrefixWithBar(req) => {
                let async_pub_sub::Request { content, response_sender } = req;
                let response = <Self as RpcInterface>::prefix_with_bar(self, content)
                    .await;
                response_sender.send(response).expect("failed to send response");
            }
            RpcInterfaceMessage::GetToto(req) => {
                let async_pub_sub::Request { content: _, response_sender } = req;
                let response = <Self as RpcInterface>::get_toto(self).await;
                response_sender.send(response).expect("failed to send response");
            }
            RpcInterfaceMessage::SetTata(req) => {
                let async_pub_sub::Request { content, response_sender } = req;
                let response = <Self as RpcInterface>::set_tata(self, content).await;
                response_sender.send(response).expect("failed to send response");
            }
        }
    }
}
impl<T> RpcInterfaceServer for T
where
    T: RpcInterface + async_pub_sub::SubscriberWrapper<RpcInterfaceMessage>,
{}
fn main() {}
