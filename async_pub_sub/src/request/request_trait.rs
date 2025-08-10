use crate::Result;

pub trait Request {
    type Response;
    fn respond(self, response: Self::Response) -> impl Future<Output = Result<()>>;
}
