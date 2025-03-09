use std::future::Future;

use futures::stream::BoxStream;

use crate::Result;

pub trait Publisher<Message>
where
    Message: Send + 'static,
{
    fn get_name(&self) -> &'static str;

    fn publish(&self, message: Message) -> impl Future<Output = Result<()>>;

    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<BoxStream<'static, Message>>;
}
