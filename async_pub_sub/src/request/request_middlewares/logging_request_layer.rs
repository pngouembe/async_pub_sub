use std::{fmt::Debug, future::Future};

use futures::{FutureExt, future::BoxFuture};

use crate::{Layer, Request, Result};

/// A request middleware layer that adds logging capabilities to any request.
/// This layer will log request content and responses for debugging purposes.
pub struct LoggingRequestLayer {
    request_name: &'static str,
}

impl LoggingRequestLayer {
    /// Creates a new logging request layer.
    ///
    /// # Arguments
    /// * `request_name` - Name to use in log messages for this request
    pub fn new(request_name: &'static str) -> Self {
        Self { request_name }
    }
}

impl<R> Layer<R> for LoggingRequestLayer
where
    R: Request + Send,
    R::Content: Debug,
    R::Response: Debug,
    R::SentResponse: Debug,
{
    type LayerType = LoggingRequest<R>;

    fn layer(self, request: R) -> Self::LayerType {
        LoggingRequest {
            request_name: self.request_name,
            request,
        }
    }
}

/// A request wrapper that adds logging functionality to an existing request.
/// Logs request content, responses, and method calls for debugging.
pub struct LoggingRequest<R>
where
    R: Request,
{
    /// The name used in log messages for this request
    request_name: &'static str,
    /// The underlying request being wrapped
    request: R,
}

impl<R> Request for LoggingRequest<R>
where
    R: Request + Send,
    R::Content: Debug,
    R::Response: Debug + 'static,
    R::SentResponse: Debug,
    Self: Sync,
{
    type Content = R::Content;
    type Response = R::Response;
    type SentResponse = R::SentResponse;

    fn take_response(&mut self) -> Option<BoxFuture<'static, Result<Self::Response>>> {
        let request_name = self.request_name;
        let response_future = self.request.take_response()?;

        log::info!("[{}] Taking response future", request_name);

        let logged_response = async move {
            let response = response_future.await;
            match &response {
                Ok(resp) => log::info!("[{}] Response received: {:?}", request_name, resp),
                Err(err) => log::error!("[{}] Response error: {}", request_name, err),
            }
            response
        }
        .boxed();

        Some(logged_response)
    }

    fn take_content(&mut self) -> Option<Self::Content> {
        let content = self.request.take_content();
        if let Some(ref content) = content {
            log::info!("[{}] Taking content: {:?}", self.request_name, content);
        }
        content
    }

    fn respond(self, response: Self::SentResponse) -> impl Future<Output = Result<()>> {
        log::info!("[{}] Responding with: {:?}", self.request_name, response);
        self.request.respond(response)
    }
}
