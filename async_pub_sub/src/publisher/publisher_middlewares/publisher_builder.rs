use crate::{Publisher, PublisherLayer};

// TODO: rework, take inspiration on the tower crate

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
    pub fn new(publisher: P) -> Self {
        Self { publisher }
    }

    pub fn with_layer<Layer>(self, layer: Layer) -> PublisherBuilder<Layer::PublisherType>
    where
        Layer: PublisherLayer<P>,
    {
        PublisherBuilder::new(layer.layer(self.publisher))
    }

    pub fn build(self) -> P {
        self.publisher
    }
}
