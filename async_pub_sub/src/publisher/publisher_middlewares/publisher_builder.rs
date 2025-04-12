use crate::{utils::IdentityLayer, Layer, Publisher};

/// A builder pattern implementation for constructing a publisher with middleware layers.
/// This struct allows for composing multiple middleware layers.
#[derive(Clone)]
pub struct PublisherBuilder<L = IdentityLayer> {
    layer: L,
}

impl Default for PublisherBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PublisherBuilder {
    /// Creates a new `PublisherBuilder` with no layers.
    pub fn new() -> Self {
        Self {
            layer: IdentityLayer::new(),
        }
    }
}

impl<L> PublisherBuilder<L> {
    /// Adds a middleware layer to the builder.
    ///
    /// # Arguments
    /// * `layer` - The middleware layer to add
    pub fn layer<NewLayer>(self, layer: NewLayer) -> PublisherBuilder<NewLayer> {
        PublisherBuilder { layer }
    }

    /// Wraps a publisher with the composed layers.
    ///
    /// # Arguments
    /// * `publisher` - The publisher to wrap with the composed layers
    pub fn publisher<P>(self, publisher: P) -> L::LayerType
    where
        P: Publisher,
        L: Layer<P>,
        L::LayerType: Publisher,
    {
        Layer::layer(&self.layer, publisher)
    }
}
