use crate::{
    Layer,
    utils::{IdentityLayer, forwarder::Forwarder},
};

/// A builder pattern implementation for constructing a forwarder with middleware layers.
/// This struct allows for composing multiple middleware layers.
#[derive(Clone)]
pub struct ForwarderBuilder<L = IdentityLayer> {
    layer: L,
}

impl Default for ForwarderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ForwarderBuilder {
    /// Creates a new `ForwarderBuilder` with no layers.
    pub fn new() -> Self {
        Self {
            layer: IdentityLayer::new(),
        }
    }
}

impl<L> ForwarderBuilder<L> {
    /// Adds a middleware layer to the builder.
    ///
    /// # Arguments
    /// * `layer` - The middleware layer to add
    pub fn layer<NewLayer>(self, layer: NewLayer) -> ForwarderBuilder<NewLayer> {
        ForwarderBuilder { layer }
    }

    /// Wraps a forwarder with the composed layers.
    ///
    /// # Arguments
    /// * `forwarder` - The forwarder to wrap with the composed layers
    pub fn forwarder<F>(self, forwarder: F) -> L::LayerType
    where
        F: Forwarder,
        L: Layer<F>,
        L::LayerType: Forwarder,
    {
        Layer::layer(&self.layer, forwarder)
    }
}
