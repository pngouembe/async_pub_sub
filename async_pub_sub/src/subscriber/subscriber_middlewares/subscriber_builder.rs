use crate::{
    utils::{IdentityLayer, Layer},
    Subscriber,
};

/// A builder pattern implementation for constructing a subscriber with middleware layers.
/// This struct allows for composing multiple middleware layers.
#[derive(Clone)]
pub struct SubscriberBuilder<L = IdentityLayer> {
    layer: L,
}

impl Default for SubscriberBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SubscriberBuilder {
    /// Creates a new `SubscriberBuilder` with no layers.
    pub fn new() -> Self {
        Self {
            layer: IdentityLayer::new(),
        }
    }
}

impl<L> SubscriberBuilder<L> {
    /// Adds a middleware layer to the builder.
    ///
    /// # Arguments
    /// * `layer` - The middleware layer to add
    pub fn layer<NewLayer>(self, layer: NewLayer) -> SubscriberBuilder<NewLayer> {
        SubscriberBuilder { layer }
    }

    /// Wraps a subscriber with the composed layers.
    ///
    /// # Arguments
    /// * `subscriber` - The subscriber to wrap with the composed layers
    pub fn subscriber<S>(self, subscriber: S) -> L::LayerType
    where
        S: Subscriber,
        L: Layer<S>,
        L::LayerType: Subscriber,
    {
        Layer::layer(&self.layer, subscriber)
    }
}
