use futures::future::BoxFuture;

use crate::{Publisher, Result};
use std::future::Future;

pub trait Request: Sized {
    type Content;
    type Response;
    type SentResponse;

    #[must_use = "response future must be awaited to receive the response"]
    fn take_response(&mut self) -> Option<BoxFuture<'static, Result<Self::Response>>>;
    fn take_content(&mut self) -> Option<Self::Content>;
    fn respond(self, response: Self::SentResponse) -> impl Future<Output = Result<()>>;
}

pub trait Requester: Publisher
where
    Self::InputMessage: Request,
{
    fn request(
        &self,
        request: Self::InputMessage,
    ) -> impl Future<Output = Result<<Self::InputMessage as Request>::Response>>;
}
