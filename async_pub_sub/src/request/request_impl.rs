use std::fmt::{Debug, Display};

use crate::Request;

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
/// # use async_pub_sub::RequestImpl;
/// # #[tokio::main]
/// # async fn main() {
/// let (request, response_receiver) = RequestImpl::new(String::from("hello"));
/// assert_eq!(request.content, "hello");
/// request.respond(42);
/// assert_eq!(response_receiver.await.unwrap(), 42);
/// # }
/// ```
pub struct RequestImpl<Req, Rsp>
where
    Req: Debug,
    Rsp: Debug,
{
    pub content: Req,
    pub response_sender: futures::channel::oneshot::Sender<Rsp>,
}

impl<Req, Rsp> RequestImpl<Req, Rsp>
where
    Req: Debug,
    Rsp: Debug,
{
    pub fn new(content: Req) -> (Self, futures::channel::oneshot::Receiver<Rsp>) {
        let (response_sender, response_receiver) = futures::channel::oneshot::channel();
        (
            Self {
                content,
                response_sender,
            },
            response_receiver,
        )
    }

    pub fn respond(self, response: Rsp) {
        self.response_sender
            .send(response)
            .expect("failed to send response");
    }
}

impl<Req, Rsp> Request for RequestImpl<Req, Rsp>
where
    Req: Debug,
    Rsp: Debug,
{
    type Response = Rsp;

    fn respond(self, response: Self::Response) -> impl Future<Output = crate::Result<()>> {
        async move {
            RequestImpl::respond(self, response);
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
            "Request({}: {})",
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
