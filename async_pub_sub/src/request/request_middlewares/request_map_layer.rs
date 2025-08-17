use std::sync::Arc;

use futures::future::BoxFuture;

use crate::{Request, Result};

pub struct MappedRequest<Inner, Req, Rsp>
where
    Inner: Request,
{
    content: Option<Req>,
    response_map_function: Arc<dyn Fn(Rsp) -> Inner::SentResponse + Send + Sync>,
    inner_request: Inner,
}

impl<Inner, Req, Rsp> MappedRequest<Inner, Req, Rsp>
where
    Inner: Request,
{
    pub fn new<F>(content: Req, map_function: F, inner_request: Inner) -> Self
    where
        F: Fn(Rsp) -> Inner::SentResponse + Send + Sync + 'static,
    {
        let response_map_function = Arc::new(map_function);
        Self {
            content: Some(content),
            response_map_function,
            inner_request,
        }
    }
}

impl<Inner, Req, Rsp> Request for MappedRequest<Inner, Req, Rsp>
where
    Rsp: 'static,
    Inner: Request,
    Inner::Response: 'static,
{
    type Content = Req;
    type Response = Inner::Response;
    type SentResponse = Rsp;

    fn take_response(&mut self) -> Option<BoxFuture<'static, Result<Self::Response>>> {
        self.inner_request.take_response()
    }

    fn take_content(&mut self) -> Option<Self::Content> {
        self.content.take()
    }

    fn respond(self, response: Self::SentResponse) -> impl Future<Output = Result<()>> {
        let response = (self.response_map_function)(response);
        self.inner_request.respond(response)
    }
}
