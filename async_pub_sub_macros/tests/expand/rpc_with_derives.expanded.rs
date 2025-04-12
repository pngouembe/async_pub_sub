#![allow(unused_imports)]
use async_pub_sub_macros::rpc_interface;
#[allow(async_fn_in_trait)]
pub trait RpcInterface {
    async fn add_one(&self, value: i32) -> i32;
    async fn prefix_with_bar(&self, string: String) -> String;
}
pub enum RpcInterfaceMessage {
    AddOne(async_pub_sub::Request<i32, i32>),
    PrefixWithBar(async_pub_sub::Request<String, String>),
}
#[automatically_derived]
impl ::core::fmt::Debug for RpcInterfaceMessage {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            RpcInterfaceMessage::AddOne(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "AddOne", &__self_0)
            }
            RpcInterfaceMessage::PrefixWithBar(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "PrefixWithBar",
                    &__self_0,
                )
            }
        }
    }
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
}
impl<T> RpcInterface for T
where
    T: RpcInterfaceClient,
{
    async fn add_one(&self, value: i32) -> i32 {
        <Self as RpcInterfaceClient>::add_one(self, value).await
    }
    async fn prefix_with_bar(&self, string: String) -> String {
        <Self as RpcInterfaceClient>::prefix_with_bar(self, string).await
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
            RpcInterfaceMessage::PrefixWithBar(req) => {
                let async_pub_sub::Request { content, response_sender } = req;
                let response = <Self as RpcInterface>::prefix_with_bar(self, content)
                    .await;
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
