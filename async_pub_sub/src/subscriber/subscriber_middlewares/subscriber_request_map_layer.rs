use std::sync::Arc;

use crate::{Layer, MappedRequest, PublisherWrapper, Request, Result, Subscriber};
use futures::FutureExt;
use futures::future::BoxFuture;

pub struct SubscriberRequestMapLayer<InReq, InRsp, OutReq, OutRsp> {
    request_map_function: Arc<dyn Fn(InReq) -> Result<OutReq> + Send + Sync>,
    response_map_function: Arc<dyn Fn(OutRsp) -> Result<InRsp> + Send + Sync>,
}

impl<InReq, InRsp, OutReq, OutRsp> SubscriberRequestMapLayer<InReq, InRsp, OutReq, OutRsp> {
    pub fn new<RF, ResF>(request_map_function: RF, response_map_function: ResF) -> Self
    where
        RF: Fn(InReq) -> Result<OutReq> + Send + Sync + 'static,
        ResF: Fn(OutRsp) -> Result<InRsp> + Send + Sync + 'static,
    {
        Self {
            request_map_function: Arc::new(request_map_function),
            response_map_function: Arc::new(response_map_function),
        }
    }
}

impl<InReq, InRsp, OutReq, OutRsp, S> Layer<S>
    for SubscriberRequestMapLayer<InReq, InRsp, OutReq, OutRsp>
where
    S: Subscriber + Send,
    S::OutputMessage:
        Request<Content = InReq, Response = InRsp, SentResponse = InRsp> + Send + 'static,
    InReq: Send + Sync + 'static,
    InRsp: Send + Sync + 'static,
    OutReq: Send + Sync + 'static,
    OutRsp: Send + Sync + 'static,
{
    type LayerType = SubscriberRequestMapSubscriber<OutReq, OutRsp, S>;

    fn layer(self, subscriber: S) -> Self::LayerType {
        SubscriberRequestMapSubscriber {
            request_map_function: self.request_map_function,
            response_map_function: self.response_map_function,
            subscriber,
        }
    }
}

pub struct SubscriberRequestMapSubscriber<Req, Rsp, S>
where
    S: Subscriber,
    S::OutputMessage: Request,
{
    request_map_function:
        Arc<dyn Fn(<S::OutputMessage as Request>::Content) -> Result<Req> + Send + Sync>,
    response_map_function:
        Arc<dyn Fn(Rsp) -> Result<<S::OutputMessage as Request>::SentResponse> + Send + Sync>,
    subscriber: S,
}

impl<Req, Rsp, S> Subscriber for SubscriberRequestMapSubscriber<Req, Rsp, S>
where
    S: Subscriber,
    S::OutputMessage: Request,
    Req: Send + 'static,
    Rsp: Send + 'static,
{
    type InputMessage = S::InputMessage;
    type OutputMessage = MappedRequest<S::OutputMessage, Req, Rsp>;

    fn get_name(&self) -> &'static str {
        self.subscriber.get_name()
    }

    fn subscribe_to<P, Input>(&mut self, publisher: &mut P) -> Result<()>
    where
        P: PublisherWrapper<Input, Self::InputMessage>,
        Input: Send + 'static,
    {
        self.subscriber.subscribe_to(publisher)
    }

    fn receive(&mut self) -> BoxFuture<'_, Self::OutputMessage> {
        let future_request = self.subscriber.receive();
        let request_map_function = self.request_map_function.clone();
        let response_map_function = self.response_map_function.clone();

        async move {
            let mut inner_request = future_request.await;
            let inner_content = inner_request
                .take_content()
                .expect("content already consumed");

            let content = (request_map_function)(inner_content).expect("request conversion failed");

            MappedRequest::new(
                content,
                move |response| {
                    (response_map_function)(response).expect("response conversion failed")
                },
                inner_request,
            )
        }
        .boxed()
    }
}

pub struct SubscriberRequestMapWrapper<InReq, InRsp, OutReq, OutRsp, R>
where
    R: Request<Content = InReq, Response = InRsp, SentResponse = InRsp>,
{
    content: Option<OutReq>,
    inner_request: Option<R>,
    response_map_function: Arc<dyn Fn(OutRsp) -> Result<InRsp> + Send + Sync>,
}

impl<InReq, InRsp, OutReq, OutRsp, R> SubscriberRequestMapWrapper<InReq, InRsp, OutReq, OutRsp, R>
where
    R: Request<Content = InReq, Response = InRsp, SentResponse = InRsp>,
{
    pub fn new(
        content: OutReq,
        inner_request: R,
        response_map_function: Arc<dyn Fn(OutRsp) -> Result<InRsp> + Send + Sync>,
    ) -> Self {
        Self {
            content: Some(content),
            inner_request: Some(inner_request),
            response_map_function,
        }
    }
}

impl<InReq, InRsp, OutReq, OutRsp, R> Request
    for SubscriberRequestMapWrapper<InReq, InRsp, OutReq, OutRsp, R>
where
    InReq: Send + 'static,
    InRsp: Send + 'static,
    OutReq: Send + 'static,
    OutRsp: Send + 'static,
    R: Request<Content = InReq, Response = InRsp, SentResponse = InRsp> + Send + 'static,
{
    type Content = OutReq;
    type Response = OutRsp;
    type SentResponse = OutRsp;

    fn take_response(&mut self) -> Option<BoxFuture<'static, Result<Self::Response>>> {
        panic!(
            "SubscriberRequestMapWrapper does not support take_response - it's for server-side handling only"
        );
    }

    async fn respond(
        mut self,
        response: Self::SentResponse,
    ) -> Result<()> {
        let inner_request = self.inner_request.take().expect("Request already used");
        let mapped_response = (self.response_map_function)(response)?;
        inner_request.respond(mapped_response).await
    }

    fn take_content(&mut self) -> Option<Self::Content> {
        self.content.take()
    }
}
