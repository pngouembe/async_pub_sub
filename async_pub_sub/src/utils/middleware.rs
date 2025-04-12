/// A trait for creating middleware layers.
/// This trait enables the creation of middleware that can wrap
/// and extend the functionality of other components.
///
/// # Type Parameters
/// * `Inner` - The type being wrapped
pub trait Layer<Inner> {
    /// The type that this layer produces
    type LayerType;

    /// Wraps an inner type with this layer
    ///
    /// # Arguments
    /// * `inner` - The inner component to wrap
    ///
    /// # Returns
    /// A new instance wrapped with this layer's functionality
    fn layer(&self, inner: Inner) -> Self::LayerType;
}

/// Layer that does not alter the pipeline.
/// This is used as the base case for layer composition.
#[derive(Clone, Copy, Debug, Default)]
pub struct IdentityLayer;

impl IdentityLayer {
    /// Creates a new identity layer.
    pub fn new() -> Self {
        Self
    }
}

impl<T> Layer<T> for IdentityLayer {
    type LayerType = T;

    fn layer(&self, inner: T) -> Self::LayerType {
        inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_layer_passes_through() {
        let value = 42;
        let layered = Layer::layer(&IdentityLayer::new(), value);
        assert_eq!(layered, value);
    }
}
