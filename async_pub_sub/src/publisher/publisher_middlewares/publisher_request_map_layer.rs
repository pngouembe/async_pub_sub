use std::fmt::Debug;
use std::pin::Pin;

use crate::{Layer, Publisher, Request, RequestImpl, Requester, Result};
use futures::future::BoxFuture;
use futures::{FutureExt, Stream};

pub struct PublisherRequestMapLayer<InReq, InRsp, OutReq, OutRsp> {
    request_map_function: Box<dyn Fn(InReq) -> Result<OutReq> + Send + Sync>,
    response_map_function: Box<dyn Fn(OutRsp) -> Result<InRsp> + Send + Sync>,
}

impl<InReq, InRsp, OutReq, OutRsp> PublisherRequestMapLayer<InReq, InRsp, OutReq, OutRsp> {
    pub fn new<RF, ResF>(request_map_function: RF, response_map_function: ResF) -> Self
    where
        RF: Fn(InReq) -> Result<OutReq> + Send + Sync + 'static,
        ResF: Fn(OutRsp) -> Result<InRsp> + Send + Sync + 'static,
    {
        Self {
            request_map_function: Box::new(request_map_function),
            response_map_function: Box::new(response_map_function),
        }
    }
}

impl<InReq, InRsp, OutReq, OutRsp, P> Layer<P>
    for PublisherRequestMapLayer<InReq, InRsp, OutReq, OutRsp>
where
    P: Publisher<InputMessage = RequestImpl<OutReq, OutRsp>> + Send,
    InReq: Send + Sync + 'static,
    InRsp: Send + Sync + 'static,
    OutReq: Debug + Send + Sync + 'static,
    OutRsp: Debug + Send + Sync + 'static,
{
    type LayerType = PublisherRequestMapPublisher<InReq, InRsp, OutReq, OutRsp, P>;

    fn layer(self, publisher: P) -> Self::LayerType {
        PublisherRequestMapPublisher {
            request_map_function: self.request_map_function,
            response_map_function: self.response_map_function,
            publisher,
        }
    }
}

pub struct PublisherRequestMapPublisher<InReq, InRsp, OutReq, OutRsp, P>
where
    P: Publisher<InputMessage = RequestImpl<OutReq, OutRsp>>,
    OutReq: Debug,
    OutRsp: Debug,
{
    request_map_function: Box<dyn Fn(InReq) -> Result<OutReq> + Send + Sync>,
    response_map_function: Box<dyn Fn(OutRsp) -> Result<InRsp> + Send + Sync>,
    publisher: P,
}

impl<InReq, InRsp, OutReq, OutRsp, P> Publisher
    for PublisherRequestMapPublisher<InReq, InRsp, OutReq, OutRsp, P>
where
    InReq: Debug + Send + Sync + 'static,
    InRsp: Debug + Send + Sync + 'static,
    OutReq: Debug + Send + Sync + 'static,
    OutRsp: Debug + Send + Sync + 'static,
    P: Publisher<InputMessage = RequestImpl<OutReq, OutRsp>> + Send + Sync + 'static,
{
    type InputMessage = RequestImpl<InReq, InRsp>;
    type OutputMessage = P::OutputMessage;

    fn get_name(&self) -> &'static str {
        self.publisher.get_name()
    }

    fn publish(&self, mut message: Self::InputMessage) -> BoxFuture<'_, Result<()>> {
        async move {
            let content = message.take_content().unwrap();
            let mapped_content = (self.request_map_function)(content)?;
            let mut request = RequestImpl::new(mapped_content);
            let response = request.take_response().unwrap();

            self.publisher.publish(request).await?;
            let response = response.await?;

            let mapped_response = (self.response_map_function)(response)?;
            message.respond(mapped_response).await?;

            Ok(())
        }
        .boxed()
    }

    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<Pin<Box<dyn Stream<Item = Self::OutputMessage> + Send + Sync + 'static>>> {
        self.publisher.get_message_stream(subscriber_name)
    }
}

impl<InReq, InRsp, OutReq, OutRsp, P> Requester
    for PublisherRequestMapPublisher<InReq, InRsp, OutReq, OutRsp, P>
where
    InReq: Debug + Send + Sync + 'static,
    InRsp: Debug + Send + Sync + 'static,
    OutReq: Debug + Send + Sync + 'static,
    OutRsp: Debug + Send + Sync + 'static,
    P: Publisher<InputMessage = RequestImpl<OutReq, OutRsp>> + Requester + Send + Sync + 'static,
{
    fn request(
        &self,
        mut request: Self::InputMessage,
    ) -> impl std::future::Future<Output = Result<<Self::InputMessage as Request>::Response>> {
        let content = request.take_content().unwrap();
        let mapped_content = (self.request_map_function)(content);

        async move {
            let inner_request = RequestImpl::new(mapped_content?);
            let response = self.publisher.request(inner_request).await?;
            let mapped_response = (self.response_map_function)(response)?;
            Ok(mapped_response)
        }
    }
}
