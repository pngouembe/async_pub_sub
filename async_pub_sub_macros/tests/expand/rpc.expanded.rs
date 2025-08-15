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
    AddOne(async_pub_sub::RequestImpl<i32, i32>),
    Add(async_pub_sub::RequestImpl<(i32, i32), i32>),
    PrefixWithBar(async_pub_sub::RequestImpl<String, String>),
    GetToto(async_pub_sub::RequestImpl<(), String>),
    SetTata(async_pub_sub::RequestImpl<String, ()>),
}
pub enum RpcInterfaceResponse {
    AddOne(i32),
    Add(i32),
    PrefixWithBar(String),
    GetToto(String),
    SetTata(()),
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
                    RpcInterfaceResponse::Add(__self_0),
                    RpcInterfaceResponse::Add(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    RpcInterfaceResponse::PrefixWithBar(__self_0),
                    RpcInterfaceResponse::PrefixWithBar(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    RpcInterfaceResponse::GetToto(__self_0),
                    RpcInterfaceResponse::GetToto(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    RpcInterfaceResponse::SetTata(__self_0),
                    RpcInterfaceResponse::SetTata(__arg1_0),
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
            RpcInterfaceResponse::Add(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Add", &__self_0)
            }
            RpcInterfaceResponse::PrefixWithBar(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "PrefixWithBar",
                    &__self_0,
                )
            }
            RpcInterfaceResponse::GetToto(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "GetToto",
                    &__self_0,
                )
            }
            RpcInterfaceResponse::SetTata(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "SetTata",
                    &__self_0,
                )
            }
        }
    }
}
impl async_pub_sub::Request for RpcInterfaceMessage {
    type Content = RpcInterfaceMessage;
    type Response = RpcInterfaceResponse;
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
            RpcInterfaceMessage::Add(request) => {
                let (request, response_future) = request.take_response();
                let response_future = async move {
                    let response = response_future.await?;
                    Ok(RpcInterfaceResponse::Add(response))
                }
                    .boxed();
                (RpcInterfaceMessage::Add(request), response_future)
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
            RpcInterfaceMessage::GetToto(request) => {
                let (request, response_future) = request.take_response();
                let response_future = async move {
                    let response = response_future.await?;
                    Ok(RpcInterfaceResponse::GetToto(response))
                }
                    .boxed();
                (RpcInterfaceMessage::GetToto(request), response_future)
            }
            RpcInterfaceMessage::SetTata(request) => {
                let (request, response_future) = request.take_response();
                let response_future = async move {
                    let response = response_future.await?;
                    Ok(RpcInterfaceResponse::SetTata(response))
                }
                    .boxed();
                (RpcInterfaceMessage::SetTata(request), response_future)
            }
        }
    }
    fn get_content(&self) -> &Self::Content {
        ::core::panicking::panic("not implemented")
    }
    fn respond(
        self,
        response: Self::Response,
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
                RpcInterfaceMessage::Add(request) => {
                    let RpcInterfaceResponse::Add(response) = response else {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!("Expected {0} response", "Add"),
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
                RpcInterfaceMessage::GetToto(request) => {
                    let RpcInterfaceResponse::GetToto(response) = response else {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!("Expected {0} response", "GetToto"),
                            );
                        };
                    };
                    request.respond(response).await
                }
                RpcInterfaceMessage::SetTata(request) => {
                    let RpcInterfaceResponse::SetTata(response) = response else {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!("Expected {0} response", "SetTata"),
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
    fn add(
        &self,
        left: i32,
        right: i32,
    ) -> async_pub_sub::futures::future::BoxFuture<i32> {
        let request = RpcInterfaceMessage::Add(
            async_pub_sub::RequestImpl::new((left, right)),
        );
        use async_pub_sub::Requester;
        let response = self.request(request);
        async_pub_sub::futures::FutureExt::boxed(async move {
            let RpcInterfaceResponse::Add(actual_response) = response
                .await
                .expect("failed to execute add request") else {
                {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "{0}: Expected {1} response",
                            "failed to execute add request", "Add",
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
    fn get_toto(&self) -> async_pub_sub::futures::future::BoxFuture<String> {
        let request = RpcInterfaceMessage::GetToto(async_pub_sub::RequestImpl::new(()));
        use async_pub_sub::Requester;
        let response = self.request(request);
        async_pub_sub::futures::FutureExt::boxed(async move {
            let RpcInterfaceResponse::GetToto(actual_response) = response
                .await
                .expect("failed to execute get_toto request") else {
                {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "{0}: Expected {1} response",
                            "failed to execute get_toto request", "GetToto",
                        ),
                    );
                };
            };
            actual_response
        })
    }
    fn set_tata(
        &mut self,
        tata: String,
    ) -> async_pub_sub::futures::future::BoxFuture<()> {
        let request = RpcInterfaceMessage::SetTata(
            async_pub_sub::RequestImpl::new(tata),
        );
        use async_pub_sub::Requester;
        let response = self.request(request);
        async_pub_sub::futures::FutureExt::boxed(async move {
            let RpcInterfaceResponse::SetTata(actual_response) = response
                .await
                .expect("failed to execute set_tata request") else {
                {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "{0}: Expected {1} response",
                            "failed to execute set_tata request", "SetTata",
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
    async fn handle_request(&mut self, request: RpcInterfaceMessage) {
        match request {
            RpcInterfaceMessage::AddOne(req) => {
                let async_pub_sub::RequestImpl { content, response_sender, .. } = req;
                let response = <Self as RpcInterface>::add_one(self, content).await;
                response_sender.send(response).expect("failed to send response");
            }
            RpcInterfaceMessage::Add(req) => {
                let async_pub_sub::RequestImpl { content, response_sender, .. } = req;
                let (left, right) = content;
                let response = <Self as RpcInterface>::add(self, left, right).await;
                response_sender.send(response).expect("failed to send response");
            }
            RpcInterfaceMessage::PrefixWithBar(req) => {
                let async_pub_sub::RequestImpl { content, response_sender, .. } = req;
                let response = <Self as RpcInterface>::prefix_with_bar(self, content)
                    .await;
                response_sender.send(response).expect("failed to send response");
            }
            RpcInterfaceMessage::GetToto(req) => {
                let async_pub_sub::RequestImpl { content: _, response_sender, .. } = req;
                let response = <Self as RpcInterface>::get_toto(self).await;
                response_sender.send(response).expect("failed to send response");
            }
            RpcInterfaceMessage::SetTata(req) => {
                let async_pub_sub::RequestImpl { content, response_sender, .. } = req;
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
