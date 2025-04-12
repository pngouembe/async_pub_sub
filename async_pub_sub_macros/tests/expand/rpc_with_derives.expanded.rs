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
pub struct RpcInterfaceClient {
    #[publisher(RpcInterfaceMessage)]
    pub publisher: Box<dyn async_pub_sub::Publisher<Message = RpcInterfaceMessage>>,
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
        P: async_pub_sub::Publisher<Message = RpcInterfaceMessage> + 'static,
    {
        Self {
            publisher: Box::new(publisher),
        }
    }
    pub fn add_one(&self, value: i32) -> futures::future::BoxFuture<i32> {
        let (request, response) = async_pub_sub::Request::new(value);
        let publish_future = self
            .publisher
            .publish(RpcInterfaceMessage::AddOne(request));
        {
            use futures::FutureExt;
            async move {
                publish_future.await.expect("failed to publish add_one request");
                response.await.expect("failed to receive add_one response")
            }
                .boxed()
        }
    }
    pub fn prefix_with_bar(&self, string: String) -> futures::future::BoxFuture<String> {
        let (request, response) = async_pub_sub::Request::new(string);
        let publish_future = self
            .publisher
            .publish(RpcInterfaceMessage::PrefixWithBar(request));
        {
            use futures::FutureExt;
            async move {
                publish_future.await.expect("failed to publish prefix_with_bar request");
                response.await.expect("failed to receive prefix_with_bar response")
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
