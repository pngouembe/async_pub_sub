use std::fmt::{Debug, Display};

pub struct Request<Req, Rsp> {
    pub content: Req,
    response_sender: tokio::sync::oneshot::Sender<Rsp>,
}

impl<Req, Rsp> Request<Req, Rsp> {
    pub fn new(content: Req) -> (Self, tokio::sync::oneshot::Receiver<Rsp>) {
        let (response_sender, response_receiver) = tokio::sync::oneshot::channel();
        (
            Self {
                content,
                response_sender,
            },
            response_receiver,
        )
    }

    pub fn respond(self, response: Rsp) {
        //todo: handle error
        let _ = self.response_sender.send(response);
    }
}

impl<Req, Rsp> Display for Request<Req, Rsp>
where
    Req: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // todo: rework the request display
        write!(
            f,
            "Request({}: {})",
            self.content,
            std::any::type_name::<Req>(),
        )
    }
}

impl<Req, Rsp> Debug for Request<Req, Rsp> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Request({}) -> {}",
            std::any::type_name::<Req>(),
            std::any::type_name::<Rsp>(),
        )
    }
}
