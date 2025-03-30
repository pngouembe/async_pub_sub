use crate::{Publisher, PublisherLayer};

// TODO: rework, take inspiration on the tower crate
/// A builder pattern implementation for constructing a publisher with middleware layers.
/// This struct allows for composing multiple middleware layers around a base publisher.
pub struct PublisherBuilder<P>
where
    P: Publisher,
{
    publisher: P,
}

impl<P> PublisherBuilder<P>
where
    P: Publisher,
{
    /// Creates a new `PublisherBuilder` with a base publisher.
    ///
    /// # Arguments
    ///
    /// * `publisher` - The base publisher implementation to wrap with middleware layers.
    pub fn new(publisher: P) -> Self {
        Self { publisher }
    }

    /// Adds a middleware layer to the publisher.
    ///
    /// # Arguments
    ///
    /// * `layer` - The middleware layer to add.
    ///
    /// # Returns
    ///
    /// Returns a new `PublisherBuilder` with the layer applied to the publisher.
    pub fn with_layer<Layer>(self, layer: Layer) -> PublisherBuilder<Layer::PublisherType>
    where
        Layer: PublisherLayer<P>,
    {
        PublisherBuilder::new(layer.layer(self.publisher))
    }

    /// Finalizes the builder and returns the constructed publisher with all applied layers.
    ///
    /// # Returns
    ///
    /// Returns the final publisher with all middleware layers applied.
    pub fn build(self) -> P {
        self.publisher
    }
}
