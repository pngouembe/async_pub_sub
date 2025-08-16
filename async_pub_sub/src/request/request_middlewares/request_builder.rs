use crate::{Layer, Request, utils::IdentityLayer};

/// A builder pattern implementation for constructing a request with middleware layers.
/// This struct allows for composing multiple middleware layers on requests.
#[derive(Clone)]
pub struct RequestBuilder<L = IdentityLayer> {
    layer: L,
}

impl Default for RequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestBuilder {
    /// Creates a new `RequestBuilder` with no layers.
    pub fn new() -> Self {
        Self {
            layer: IdentityLayer::new(),
        }
    }
}

impl<L> RequestBuilder<L> {
    /// Adds a middleware layer to the builder.
    ///
    /// # Arguments
    /// * `layer` - The middleware layer to add
    pub fn layer<NewLayer>(self, layer: NewLayer) -> RequestBuilder<NewLayer> {
        RequestBuilder { layer }
    }

    /// Wraps a request with the composed layers.
    ///
    /// # Arguments
    /// * `request` - The request to wrap with the composed layers
    pub fn request<R>(self, request: R) -> L::LayerType
    where
        R: Request,
        L: Layer<R>,
        L::LayerType: Request,
    {
        Layer::layer(self.layer, request)
    }
}