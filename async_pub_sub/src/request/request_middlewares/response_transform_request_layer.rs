use crate::{Layer, Request, request::ResponseTransformRequest};

/// A request middleware layer that enables response type transformation.
/// This layer wraps requests to transform the response type returned by `take_response()`
/// while preserving the original `SentResponse` type for `respond()`.
pub struct ResponseTransformRequestLayer<F, TransformedResponse> {
    transform_fn: F,
    _phantom: std::marker::PhantomData<TransformedResponse>,
}

impl<F, TransformedResponse> ResponseTransformRequestLayer<F, TransformedResponse> {
    /// Creates a new response transform layer.
    ///
    /// # Arguments
    /// * `transform_fn` - Function to transform the response type
    pub fn new(transform_fn: F) -> Self {
        Self {
            transform_fn,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<R, F, TransformedResponse> Layer<R> for ResponseTransformRequestLayer<F, TransformedResponse>
where
    R: Request,
    R::Response: 'static,
    TransformedResponse: 'static,
    F: FnOnce(R::Response) -> TransformedResponse + Send + Sync + 'static,
{
    type LayerType = ResponseTransformRequest<R, TransformedResponse>;

    fn layer(self, request: R) -> Self::LayerType {
        ResponseTransformRequest::new(request, self.transform_fn)
    }
}