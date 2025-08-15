use futures::future::BoxFuture;

use crate::{Publisher, Result};
use std::future::Future;

pub trait Request: Sized {
    type Content;
    type Response;

    #[must_use = "response future must be awaited to receive the response"]
    fn take_response(self) -> (Self, BoxFuture<'static, Result<Self::Response>>);
    fn get_content(&self) -> &Self::Content;
    fn respond(self, response: Self::Response) -> impl Future<Output = Result<()>>;
}

pub trait Requester: Publisher
where
    Self::Message: Request,
{
    fn request(
        &self,
        request: Self::Message,
    ) -> impl Future<Output = Result<<Self::Message as Request>::Response>>;
}
