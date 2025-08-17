use std::fmt::Debug;

use async_pub_sub::{
    Layer, Publisher, PublisherRequestMapLayer, PublisherRequestMapPublisher, RequestImpl,
};
use bytes::Bytes;

pub struct SerdeRequestSerializationLayer<Req, Rsp> {
    inner: PublisherRequestMapLayer<Req, Rsp, Bytes, Bytes>,
}

pub type SerdeRequestSerializationPublisher<Req, Rsp, P> =
    PublisherRequestMapPublisher<Req, Rsp, Bytes, Bytes, P>;

impl<Req, Rsp> SerdeRequestSerializationLayer<Req, Rsp>
where
    Req: Debug + Send + Sync + 'static,
    Rsp: Debug + Send + Sync + 'static,
{
    pub fn new<SF, DF, SErr, DErr, SOut>(
        serialization_function: SF,
        deserialization_function: DF,
    ) -> Self
    where
        SF: Fn(&Req) -> std::result::Result<SOut, SErr> + Send + Sync + 'static,
        DF: for<'a> Fn(&'a [u8]) -> std::result::Result<Rsp, DErr> + Send + Sync + 'static,
        SErr: Into<async_pub_sub::Error> + Send + Sync + 'static,
        DErr: Into<async_pub_sub::Error> + Send + Sync + 'static,
        SOut: Into<Bytes>,
    {
        let request_map = move |req: Req| {
            serialization_function(&req)
                .map(|out| out.into())
                .map_err(|err| err.into())
        };
        let response_map =
            move |bytes: Bytes| deserialization_function(&bytes).map_err(|err| err.into());

        Self {
            inner: PublisherRequestMapLayer::new(request_map, response_map),
        }
    }

    pub fn serde_json() -> Self
    where
        Req: serde::Serialize + Debug + Send + Sync + 'static,
        Rsp: serde::de::DeserializeOwned + Debug + Send + Sync + 'static,
    {
        let request_map = |req: Req| {
            serde_json::to_vec(&req)
                .map(Bytes::from)
                .map_err(async_pub_sub::Error::from)
        };
        let response_map = |bytes: Bytes| {
            serde_json::from_slice(&bytes).map_err(async_pub_sub::Error::from)
        };

        Self {
            inner: PublisherRequestMapLayer::new(request_map, response_map),
        }
    }
}

impl<Req, Rsp, P> Layer<P> for SerdeRequestSerializationLayer<Req, Rsp>
where
    Req: Debug + Send + Sync + 'static,
    Rsp: Debug + Send + Sync + 'static,
    P: Publisher<InputMessage = RequestImpl<Bytes, Bytes>> + Send,
{
    type LayerType = SerdeRequestSerializationPublisher<Req, Rsp, P>;

    fn layer(self, publisher: P) -> Self::LayerType {
        self.inner.layer(publisher)
    }
}
