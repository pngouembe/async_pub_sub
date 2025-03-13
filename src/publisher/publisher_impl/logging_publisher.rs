use std::{fmt::Display, pin::Pin};

use futures::{future::BoxFuture, FutureExt, Stream};

use crate::{Publisher, Request, Result, SimplePublisher};

pub struct LoggingPublisher<Message>
where
    Message: Display + Send + 'static,
{
    subscriber_name: Option<&'static str>,
    publisher: SimplePublisher<Message>,
}

impl<Message> LoggingPublisher<Message>
where
    Message: Display + Send + 'static,
{
    pub fn new(name: &'static str, buffer_size: usize) -> Self {
        Self {
            subscriber_name: None,
            publisher: SimplePublisher::new(name, buffer_size),
        }
    }
}

impl<Message> Publisher<Message> for LoggingPublisher<Message>
where
    Message: Display + Send + 'static,
{
    fn get_name(&self) -> &'static str {
        self.publisher.get_name()
    }

    fn publish_event(&self, message: Message) -> BoxFuture<Result<()>> {
        async move {
            let message_str = format!("{}", &message);
            let result = self.publisher.publish(message).await;
            log::info!(
                "[{}] -> [{}]: {}",
                self.publisher.get_name(),
                self.subscriber_name
                    .expect("subscriber name should be known"),
                message_str
            );
            result
        }
        .boxed()
    }

    fn publish_request<Req, Rsp>(&self, request: Req) -> BoxFuture<Result<Rsp>>
    where
        Req: Display + Send + 'static,
        Rsp: Display + Send + 'static,
        Message: From<Request<Req, Rsp>>,
    {
        let (request, response) = Request::<Req, Rsp>::new(request);
        let request_str = format!("{}", &request);
        let publisher_name = self.publisher.get_name();
        let subscriber_name = self
            .subscriber_name
            .expect("subscriber name should be known");

        async move {
            self.publisher.publish(request.into()).await.unwrap();

            log::info!(
                "[{}] -> [{}]: {}",
                publisher_name,
                subscriber_name,
                request_str
            );

            let response = response.await.unwrap();

            log::info!(
                "[{}] <- [{}]: {} -> {}",
                publisher_name,
                subscriber_name,
                request_str,
                response
            );

            Ok(response)
        }
        .boxed()
    }

    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<Pin<Box<dyn Stream<Item = Message> + Send + Sync + 'static>>> {
        let stream = self.publisher.get_message_stream(subscriber_name)?;
        self.subscriber_name = Some(subscriber_name);
        log::info!(
            "({}) <-> ({}): {}",
            self.publisher.get_name(),
            subscriber_name,
            std::any::type_name::<Message>()
        );
        Ok(stream)
    }
}
