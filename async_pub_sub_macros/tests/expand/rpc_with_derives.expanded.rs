#![allow(unused_imports)]
use async_pub_sub_macros::rpc_interface;
#[allow(async_fn_in_trait)]
pub trait RpcInterface {
    async fn add_one(&self, value: i32) -> i32;
    async fn prefix_with_bar(&self, string: String) -> String;
}
pub enum RpcInterfaceContent {
    AddOne(i32),
    PrefixWithBar(String),
}
#[automatically_derived]
impl ::core::fmt::Debug for RpcInterfaceContent {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            RpcInterfaceContent::AddOne(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "AddOne", &__self_0)
            }
            RpcInterfaceContent::PrefixWithBar(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "PrefixWithBar",
                    &__self_0,
                )
            }
        }
    }
}
pub enum RpcInterfaceResponse {
    AddOne(i32),
    PrefixWithBar(String),
}
#[automatically_derived]
impl ::core::fmt::Debug for RpcInterfaceResponse {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            RpcInterfaceResponse::AddOne(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "AddOne", &__self_0)
            }
            RpcInterfaceResponse::PrefixWithBar(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "PrefixWithBar",
                    &__self_0,
                )
            }
        }
    }
}
pub type RpcInterfaceMessage = async_pub_sub::RequestImpl<
    RpcInterfaceContent,
    RpcInterfaceResponse,
>;
pub struct RpcInterfaceClient<OutputMessage = RpcInterfaceMessage>
where
    OutputMessage: Send + 'static,
{
    pub publisher: Box<
        dyn async_pub_sub::Publisher<
            InputMessage = RpcInterfaceMessage,
            OutputMessage = OutputMessage,
        > + Send,
    >,
}
impl<OutputMessage> async_pub_sub::Publisher for RpcInterfaceClient<OutputMessage>
where
    OutputMessage: Send + 'static,
{
    type InputMessage = RpcInterfaceMessage;
    type OutputMessage = OutputMessage;
    fn get_name(&self) -> &'static str {
        async_pub_sub::Publisher::get_name(&self.publisher)
    }
    fn publish(
        &self,
        message: Self::InputMessage,
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
                    Item = Self::OutputMessage,
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
impl<OutputMessage> RpcInterfaceClient<OutputMessage>
where
    OutputMessage: Send + 'static,
{
    pub fn new<P>(publisher: P) -> Self
    where
        P: async_pub_sub::Publisher<
                InputMessage = RpcInterfaceMessage,
                OutputMessage = OutputMessage,
            > + Send + 'static,
    {
        Self {
            publisher: Box::new(publisher),
        }
    }
}
impl<OutputMessage> async_pub_sub::Requester for RpcInterfaceClient<OutputMessage>
where
    OutputMessage: Send + 'static,
    RpcInterfaceMessage: async_pub_sub::Request,
{
    fn request(
        &self,
        mut request: Self::InputMessage,
    ) -> impl std::future::Future<
        Output = async_pub_sub::Result<
            <Self::InputMessage as async_pub_sub::Request>::Response,
        >,
    > {
        use async_pub_sub::Request;
        let response = request.take_response().expect("failed to get response future");
        let publication_future = self.publisher.publish(request);
        async move {
            publication_future.await?;
            response.await
        }
    }
}
impl<OutputMessage> RpcInterface for RpcInterfaceClient<OutputMessage>
where
    OutputMessage: Send + 'static,
{
    fn add_one(&self, value: i32) -> async_pub_sub::futures::future::BoxFuture<'_, i32> {
        let content = RpcInterfaceContent::AddOne(value);
        let request = async_pub_sub::RequestImpl::new(content);
        use async_pub_sub::Requester;
        let response = self.request(request);
        async_pub_sub::futures::FutureExt::boxed(async move {
            let RpcInterfaceResponse::AddOne(actual_response) = response
                .await
                .expect("failed to execute add_one request") else {
                {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "{0}: Expected {1} response",
                            "failed to execute add_one request", "AddOne",
                        ),
                    );
                };
            };
            actual_response
        })
    }
    fn prefix_with_bar(
        &self,
        string: String,
    ) -> async_pub_sub::futures::future::BoxFuture<'_, String> {
        let content = RpcInterfaceContent::PrefixWithBar(string);
        let request = async_pub_sub::RequestImpl::new(content);
        use async_pub_sub::Requester;
        let response = self.request(request);
        async_pub_sub::futures::FutureExt::boxed(async move {
            let RpcInterfaceResponse::PrefixWithBar(actual_response) = response
                .await
                .expect("failed to execute prefix_with_bar request") else {
                {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "{0}: Expected {1} response",
                            "failed to execute prefix_with_bar request", "PrefixWithBar",
                        ),
                    );
                };
            };
            actual_response
        })
    }
}
pub trait RpcInterfaceServer<
    Input,
    Output,
>: RpcInterface + async_pub_sub::SubscriberWrapper<Input, Output>
where
    Input: Send + 'static,
    Output: async_pub_sub::Request<
            Content = RpcInterfaceContent,
            SentResponse = RpcInterfaceResponse,
        > + Send + 'static,
    Output::Response: Send,
{
    async fn run(&mut self) {
        loop {
            let request = self.receive().await;
            self.handle_request(request).await;
        }
    }
    async fn handle_request(&mut self, mut req: Output) {
        use async_pub_sub::Request;
        let content = req.take_content().expect("failed to get content");
        match content {
            RpcInterfaceContent::AddOne(params) => {
                let response = <Self as RpcInterface>::add_one(self, params).await;
                let response_content = RpcInterfaceResponse::AddOne(response);
                req.respond(response_content).await.expect("failed to send response");
            }
            RpcInterfaceContent::PrefixWithBar(params) => {
                let response = <Self as RpcInterface>::prefix_with_bar(self, params)
                    .await;
                let response_content = RpcInterfaceResponse::PrefixWithBar(response);
                req.respond(response_content).await.expect("failed to send response");
            }
        }
    }
}
impl<Input, Output, T> RpcInterfaceServer<Input, Output> for T
where
    T: RpcInterface + async_pub_sub::SubscriberWrapper<Input, Output>,
    Input: Send + 'static,
    Output: async_pub_sub::Request<
            Content = RpcInterfaceContent,
            SentResponse = RpcInterfaceResponse,
        > + Send + 'static,
    Output::Response: Send,
{}
fn main() {}
