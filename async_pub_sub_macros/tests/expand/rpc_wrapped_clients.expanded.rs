#![allow(unused_imports)]
use async_pub_sub_macros::rpc_interface;
#[allow(async_fn_in_trait)]
pub trait TestRpc {
    async fn add_one(&self, value: i32) -> i32;
}
pub enum TestRpcContent {
    AddOne(i32),
}
#[automatically_derived]
impl ::core::fmt::Debug for TestRpcContent {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            TestRpcContent::AddOne(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "AddOne", &__self_0)
            }
        }
    }
}
pub enum TestRpcResponse {
    AddOne(i32),
}
#[automatically_derived]
impl ::core::fmt::Debug for TestRpcResponse {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            TestRpcResponse::AddOne(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "AddOne", &__self_0)
            }
        }
    }
}
pub type TestRpcMessage = async_pub_sub::RequestImpl<TestRpcContent, TestRpcResponse>;
pub struct TestRpcClient<OutputMessage = TestRpcMessage>
where
    OutputMessage: Send + 'static,
{
    pub publisher: Box<
        dyn async_pub_sub::Publisher<
            InputMessage = TestRpcMessage,
            OutputMessage = OutputMessage,
        > + Send,
    >,
}
impl<OutputMessage> async_pub_sub::Publisher for TestRpcClient<OutputMessage>
where
    OutputMessage: Send + 'static,
{
    type InputMessage = TestRpcMessage;
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
impl<OutputMessage> TestRpcClient<OutputMessage>
where
    OutputMessage: Send + 'static,
{
    pub fn new<P>(publisher: P) -> Self
    where
        P: async_pub_sub::Publisher<
                InputMessage = TestRpcMessage,
                OutputMessage = OutputMessage,
            > + Send + 'static,
    {
        Self {
            publisher: Box::new(publisher),
        }
    }
}
impl<OutputMessage> async_pub_sub::Requester for TestRpcClient<OutputMessage>
where
    OutputMessage: Send + 'static,
    TestRpcMessage: async_pub_sub::Request,
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
impl<OutputMessage> TestRpc for TestRpcClient<OutputMessage>
where
    OutputMessage: Send + 'static,
{
    fn add_one(&self, value: i32) -> async_pub_sub::futures::future::BoxFuture<'_, i32> {
        let content = TestRpcContent::AddOne(value);
        let request = async_pub_sub::RequestImpl::new(content);
        use async_pub_sub::Requester;
        let response = self.request(request);
        async_pub_sub::futures::FutureExt::boxed(async move {
            let TestRpcResponse::AddOne(actual_response) = response
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
}
pub trait TestRpcServer<
    Input,
    Output,
>: TestRpc + async_pub_sub::SubscriberWrapper<Input, Output>
where
    Input: Send + 'static,
    Output: async_pub_sub::Request<
            Content = TestRpcContent,
            SentResponse = TestRpcResponse,
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
            TestRpcContent::AddOne(params) => {
                let response = <Self as TestRpc>::add_one(self, params).await;
                let response_content = TestRpcResponse::AddOne(response);
                req.respond(response_content).await.expect("failed to send response");
            }
        }
    }
}
impl<Input, Output, T> TestRpcServer<Input, Output> for T
where
    T: TestRpc + async_pub_sub::SubscriberWrapper<Input, Output>,
    Input: Send + 'static,
    Output: async_pub_sub::Request<
            Content = TestRpcContent,
            SentResponse = TestRpcResponse,
        > + Send + 'static,
    Output::Response: Send,
{}
pub struct ServiceWithDefaultClient {
    pub client: TestRpcClient,
    pub _name: String,
}
pub struct ServiceWithGenericClient<T>
where
    T: Send + 'static,
{
    pub client: TestRpcClient<T>,
    pub _name: String,
}
pub struct ServiceWithStringClient {
    pub client: TestRpcClient<String>,
    pub _name: String,
}
fn main() {}
