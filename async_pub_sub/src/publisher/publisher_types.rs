use std::fmt::{Debug, Display};

pub struct Request<Req, Rsp>
where
    Req: Debug,
    Rsp: Debug,
{
    pub content: Req,
    pub response_sender: futures::channel::oneshot::Sender<Rsp>,
}

impl<Req, Rsp> Request<Req, Rsp>
where
    Req: Debug,
    Rsp: Debug,
{
    pub fn new(content: Req) -> (Self, futures::channel::oneshot::Receiver<Rsp>) {
        let (response_sender, response_receiver) = futures::channel::oneshot::channel();
        (
            Self {
                content,
                response_sender,
            },
            response_receiver,
        )
    }

    pub fn respond(self, response: Rsp) {
        self.response_sender
            .send(response)
            .expect("failed to send response");
    }
}

impl<Req, Rsp> Display for Request<Req, Rsp>
where
    Req: Display + Debug,
    Rsp: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: rework the request display
        write!(
            f,
            "Request({}: {})",
            self.content,
            std::any::type_name::<Req>(),
        )
    }
}

impl<Req, Rsp> Debug for Request<Req, Rsp>
where
    Req: Debug,
    Rsp: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "inputs: {:?}", self.content)
    }
}
