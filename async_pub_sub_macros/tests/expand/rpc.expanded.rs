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
pub trait RpcInterfaceClient: async_pub_sub::PublisherWrapper<RpcInterfaceMessage> {
    fn add_one(&self, value: i32) -> impl std::future::Future<Output = i32> {
        async move {
            let (request, response) = async_pub_sub::Request::new(value);
            self.publish(RpcInterfaceMessage::AddOne(request))
                .await
                .expect("failed to publish add_one request");
            response.await.expect("failed to receive add_one response")
        }
    }
    fn add(&self, left: i32, right: i32) -> impl std::future::Future<Output = i32> {
        async move {
            let (request, response) = async_pub_sub::Request::new((left, right));
            self.publish(RpcInterfaceMessage::Add(request))
                .await
                .expect("failed to publish add request");
            response.await.expect("failed to receive add response")
        }
    }
    fn prefix_with_bar(
        &self,
        string: String,
    ) -> impl std::future::Future<Output = String> {
        async move {
            let (request, response) = async_pub_sub::Request::new(string);
            self.publish(RpcInterfaceMessage::PrefixWithBar(request))
                .await
                .expect("failed to publish prefix_with_bar request");
            response.await.expect("failed to receive prefix_with_bar response")
        }
    }
    fn get_toto(&self) -> impl std::future::Future<Output = String> {
        async move {
            let (request, response) = async_pub_sub::Request::new(());
            self.publish(RpcInterfaceMessage::GetToto(request))
                .await
                .expect("failed to publish get_toto request");
            response.await.expect("failed to receive get_toto response")
        }
    }
    fn set_tata(&mut self, tata: String) -> impl std::future::Future<Output = ()> {
        async move {
            let (request, response) = async_pub_sub::Request::new(tata);
            self.publish(RpcInterfaceMessage::SetTata(request))
                .await
                .expect("failed to publish set_tata request");
            response.await.expect("failed to receive set_tata response")
        }
    }
}
impl<T> RpcInterface for T
where
    T: RpcInterfaceClient,
{
    async fn add_one(&self, value: i32) -> i32 {
        <Self as RpcInterfaceClient>::add_one(self, value).await
    }
    async fn add(&self, left: i32, right: i32) -> i32 {
        <Self as RpcInterfaceClient>::add(self, left, right).await
    }
    async fn prefix_with_bar(&self, string: String) -> String {
        <Self as RpcInterfaceClient>::prefix_with_bar(self, string).await
    }
    async fn get_toto(&self) -> String {
        <Self as RpcInterfaceClient>::get_toto(self).await
    }
    async fn set_tata(&mut self, tata: String) {
        <Self as RpcInterfaceClient>::set_tata(self, tata).await
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
