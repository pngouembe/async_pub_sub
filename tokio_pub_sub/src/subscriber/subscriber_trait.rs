use std::future::Future;

use crate::{Publisher, Result};

pub trait Subscriber {
    type Message: Send + 'static;

    fn get_name(&self) -> &'static str;

    fn subscribe_to(
        &mut self,
        publisher: &mut impl Publisher<Message = Self::Message>,
    ) -> Result<()>;

    fn receive(&mut self) -> impl Future<Output = Self::Message>;
}
