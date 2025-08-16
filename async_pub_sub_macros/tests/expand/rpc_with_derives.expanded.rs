#![allow(unused_imports)]
use async_pub_sub_macros::rpc_interface;
#[allow(async_fn_in_trait)]
pub trait RpcInterface {
    async fn add_one(&self, value: i32) -> i32;
    async fn prefix_with_bar(&self, string: String) -> String;
}
pub enum RpcInterfaceMessage {
    AddOne(async_pub_sub::RequestImpl<i32, i32>),
    PrefixWithBar(async_pub_sub::RequestImpl<String, String>),
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
pub enum RpcInterfaceResponse {
    AddOne(i32),
    PrefixWithBar(String),
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for RpcInterfaceResponse {}
#[automatically_derived]
impl ::core::cmp::PartialEq for RpcInterfaceResponse {
    #[inline]
    fn eq(&self, other: &RpcInterfaceResponse) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (
                    RpcInterfaceResponse::AddOne(__self_0),
                    RpcInterfaceResponse::AddOne(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    RpcInterfaceResponse::PrefixWithBar(__self_0),
                    RpcInterfaceResponse::PrefixWithBar(__arg1_0),
                ) => __self_0 == __arg1_0,
                _ => unsafe { ::core::intrinsics::unreachable() }
            }
    }
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
impl async_pub_sub::Request for RpcInterfaceMessage {
    type Content = RpcInterfaceMessage;
    type Response = RpcInterfaceResponse;
    type SentResponse = RpcInterfaceResponse;
    fn take_response(
        self,
    ) -> (
        Self,
        async_pub_sub::futures::future::BoxFuture<
            'static,
            async_pub_sub::Result<Self::Response>,
        >,
    ) {
        use async_pub_sub::futures::FutureExt;
        match self {
            RpcInterfaceMessage::AddOne(request) => {
                let (request, response_future) = request.take_response();
                let response_future = async move {
                    let response = response_future.await?;
                    Ok(RpcInterfaceResponse::AddOne(response))
                }
                    .boxed();
                (RpcInterfaceMessage::AddOne(request), response_future)
            }
            RpcInterfaceMessage::PrefixWithBar(request) => {
                let (request, response_future) = request.take_response();
                let response_future = async move {
                    let response = response_future.await?;
                    Ok(RpcInterfaceResponse::PrefixWithBar(response))
                }
                    .boxed();
                (RpcInterfaceMessage::PrefixWithBar(request), response_future)
            }
        }
    }
    fn take_content(&mut self) -> Option<Self::Content> {
        ::core::panicking::panic("not implemented")
    }
    fn respond(
        self,
        response: Self::SentResponse,
    ) -> impl std::future::Future<Output = async_pub_sub::Result<()>> {
        async move {
            match self {
                RpcInterfaceMessage::AddOne(request) => {
                    let RpcInterfaceResponse::AddOne(response) = response else {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!("Expected {0} response", "AddOne"),
                            );
                        };
                    };
                    request.respond(response).await
                }
                RpcInterfaceMessage::PrefixWithBar(request) => {
                    let RpcInterfaceResponse::PrefixWithBar(response) = response else {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!("Expected {0} response", "PrefixWithBar"),
                            );
                        };
                    };
                    request.respond(response).await
                }
            }
        }
    }
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
impl async_pub_sub::Requester for RpcInterfaceClient
where
    RpcInterfaceMessage: async_pub_sub::Request,
{
    fn request(
        &self,
        request: Self::Message,
    ) -> impl std::future::Future<
        Output = async_pub_sub::Result<
            <Self::Message as async_pub_sub::Request>::Response,
        >,
    > {
        use async_pub_sub::Request;
        let (request, response) = request.take_response();
        let publication_future = self.publisher.publish(request);
        async move {
            publication_future.await?;
            response.await
        }
    }
}
impl RpcInterface for RpcInterfaceClient {
    fn add_one(&self, value: i32) -> async_pub_sub::futures::future::BoxFuture<i32> {
        let request = RpcInterfaceMessage::AddOne(
            async_pub_sub::RequestImpl::new(value),
        );
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
    ) -> async_pub_sub::futures::future::BoxFuture<String> {
        let request = RpcInterfaceMessage::PrefixWithBar(
            async_pub_sub::RequestImpl::new(string),
        );
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
pub trait RpcInterfaceServer: async_pub_sub::SubscriberWrapper<
        RpcInterfaceMessage,
    > + RpcInterface {
    async fn run(&mut self) {
        loop {
            let request = self.receive().await;
            self.handle_request(request).await;
        }
    }
    async fn handle_request(&mut self, mut request: RpcInterfaceMessage) {
        match request {
            RpcInterfaceMessage::AddOne(mut req) => {
                use async_pub_sub::Request;
                let content = req.take_content().expect("failed to get content");
                let response = <Self as RpcInterface>::add_one(self, content).await;
                req.respond(response).await.expect("failed to send response");
            }
            RpcInterfaceMessage::PrefixWithBar(mut req) => {
                use async_pub_sub::Request;
                let content = req.take_content().expect("failed to get content");
                let response = <Self as RpcInterface>::prefix_with_bar(self, content)
                    .await;
                req.respond(response).await.expect("failed to send response");
            }
        }
    }
}
impl<T> RpcInterfaceServer for T
where
    T: RpcInterface + async_pub_sub::SubscriberWrapper<RpcInterfaceMessage>,
{}
fn main() {}
