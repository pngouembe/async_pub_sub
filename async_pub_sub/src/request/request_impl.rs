use std::fmt::{Debug, Display};

use futures::{FutureExt, future::BoxFuture};

use crate::{Request, Result};

/// A request structure that represents a request-response pattern for asynchronous communication.
///
/// This struct encapsulates a request of type `Req` and provides a mechanism to send back
/// a response of type `Rsp` using a oneshot channel.
///
/// # Type Parameters
///
/// * `Req` - The request content type, must implement `Debug`
/// * `Rsp` - The response type, must implement `Debug`
///
/// # Fields
///
/// * `content` - The actual request content
/// * `response_sender` - A oneshot channel sender for sending the response
///
/// # Examples
///
/// ```
/// use async_pub_sub::{RequestImpl, Request};
/// # #[tokio::main]
/// # async fn main() {
/// let (request, response_receiver) = RequestImpl::new(String::from("hello")).take_response();
/// assert_eq!(request.content, Some("hello".to_string()));
/// request.respond(42).await.unwrap();
/// assert_eq!(response_receiver.await.unwrap(), 42);
/// # }
/// ```
pub struct RequestImpl<Req, Rsp>
where
    Req: Debug,
    Rsp: Debug,
{
    pub content: Option<Req>,
    pub response_receiver: Option<futures::channel::oneshot::Receiver<Rsp>>,
    pub response_sender: futures::channel::oneshot::Sender<Rsp>,
}

impl<Req, Rsp> RequestImpl<Req, Rsp>
where
    Req: Debug,
    Rsp: Debug,
{
    pub fn new(content: Req) -> Self {
        let (response_sender, response_receiver) = futures::channel::oneshot::channel();

        Self {
            content: Some(content),
            response_receiver: Some(response_receiver),
            response_sender,
        }
    }
}

impl<Req, Rsp> Request for RequestImpl<Req, Rsp>
where
    Req: Debug,
    Rsp: Debug + Send + 'static,
{
    type Content = Req;
    type Response = Rsp;
    type SentResponse = Rsp;

    fn take_response(mut self) -> (Self, BoxFuture<'static, Result<Self::Response>>) {
        let Some(response_receiver) = self.response_receiver.take() else {
            return (
                self,
                async move { Err("response channel closed".into()) }.boxed(),
            );
        };
        let future = async move {
            response_receiver
                .await
                .map_err(|e| format!("failed to receive response: {}", e).into())
        }
        .boxed();
        (self, future)
    }

    fn take_content(&mut self) -> Option<Self::Content> {
        self.content.take()
    }

    fn respond(self, response: Self::SentResponse) -> impl Future<Output = crate::Result<()>> {
        async move {
            self.response_sender
                .send(response)
                .expect("failed to send response");
            Ok(())
        }
    }
}

impl<Req, Rsp> Display for RequestImpl<Req, Rsp>
where
    Req: Display + Debug,
    Rsp: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: rework the request display
        write!(
            f,
            "Request({:?}: {})",
            self.content,
            std::any::type_name::<Req>(),
        )
    }
}

impl<Req, Rsp> Debug for RequestImpl<Req, Rsp>
where
    Req: Debug,
    Rsp: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "inputs: {:?}", self.content)
    }
}

/// A request wrapper that enables response type transformation.
/// This wrapper allows transforming the response type returned by `take_response()`
/// while preserving the original request's `SentResponse` type for `respond()`.
pub struct ResponseTransformRequest<Inner, TransformedResponse>
where
    Inner: Request,
{
    inner_request: Inner,
    response_conversion_function: Option<Box<dyn FnOnce(Inner::Response) -> TransformedResponse + Send + Sync>>,
}

impl<Inner, TransformedResponse> ResponseTransformRequest<Inner, TransformedResponse>
where
    Inner: Request,
{
    /// Creates a new response transform request wrapper.
    ///
    /// # Arguments
    /// * `inner_request` - The inner request to wrap
    /// * `transform_fn` - Function to transform the response type
    pub fn new<F>(inner_request: Inner, transform_fn: F) -> Self
    where
        F: FnOnce(Inner::Response) -> TransformedResponse + Send + Sync + 'static,
    {
        Self {
            inner_request,
            response_conversion_function: Some(Box::new(transform_fn)),
        }
    }
}

impl<Inner, TransformedResponse> Request for ResponseTransformRequest<Inner, TransformedResponse>
where
    Inner: Request,
    Inner::Response: 'static,
    TransformedResponse: 'static,
{
    type Content = Inner::Content;
    type Response = TransformedResponse;
    type SentResponse = Inner::SentResponse;

    fn take_response(mut self) -> (Self, BoxFuture<'static, Result<Self::Response>>) {
        let (inner_request, response) = self.inner_request.take_response();
        let response_conversion_function = self.response_conversion_function.take()
            .expect("response conversion function should be available");

        let response = async move {
            let response: Inner::Response = response.await?;
            Ok((response_conversion_function)(response))
        }
        .boxed();

        (
            Self {
                inner_request,
                response_conversion_function: None,
            },
            response,
        )
    }

    fn take_content(&mut self) -> Option<Self::Content> {
        self.inner_request.take_content()
    }

    fn respond(self, response: Self::SentResponse) -> impl Future<Output = Result<()>> {
        self.inner_request.respond(response)
    }
}
