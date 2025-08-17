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
/// let mut request = RequestImpl::new(String::from("hello"));
/// let response = request.take_response().unwrap();
/// assert_eq!(request.content, Some("hello".to_string()));
/// request.respond(42).await.unwrap();
/// assert_eq!(response.await.unwrap(), 42);
/// # }
/// ```
pub struct RequestImpl<Req, Rsp> {
    pub content: Option<Req>,
    pub response_receiver: Option<futures::channel::oneshot::Receiver<Rsp>>,
    pub response_sender: futures::channel::oneshot::Sender<Rsp>,
}

impl<Req, Rsp> RequestImpl<Req, Rsp> {
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
    Rsp: Send + 'static,
{
    type Content = Req;
    type Response = Rsp;
    type SentResponse = Rsp;

    fn take_response(&mut self) -> Option<BoxFuture<'static, Result<Self::Response>>> {
        let response_receiver = self.response_receiver.take()?;
        let future_response = async move {
            response_receiver
                .await
                .map_err(|e| format!("failed to receive response: {}", e).into())
        }
        .boxed();
        Some(future_response)
    }

    fn take_content(&mut self) -> Option<Self::Content> {
        self.content.take()
    }

    async fn respond(self, response: Self::SentResponse) -> crate::Result<()> {
        self.response_sender.send(response).map_err(|_| {
            format!(
                "Failed to send response (type: {})",
                std::any::type_name::<Rsp>()
            )
        })?;
        Ok(())
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
