use tokio_pub_sub::{Publisher, Request, Subscriber};
use tokio_pub_sub_macros::rpc_interface;
pub trait Calculator {
    type RpcMessage: Send + 'static;
    async fn add(&self, a: i32, b: i32) -> i32;
    async fn multiply(&self, a: i32, b: i32) -> i32;
}
pub struct CalculatorServer<S: Subscriber> {
    pub subscriber: S,
}
impl<S: Subscriber> Calculator for CalculatorServer<S> {
    type RpcMessage = <S as Subscriber>::Message;
    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
    async fn multiply(&self, a: i32, b: i32) -> i32 {
        a * b
    }
    async fn stringify(&self, a: i32) -> String {
        a.to_string()
    }
}
pub enum CalculatorFunctions {
    Add(tokio_pub_sub::Request<i32, i32>),
    Multiply(tokio_pub_sub::Request<i32, i32>),
    Stringify(tokio_pub_sub::Request<i32, String>),
}
impl<S> CalculatorServer<S>
where
    S: Subscriber<Message = CalculatorFunctions>,
{
    pub async fn run(&mut self) {
        loop {
            let request = self.subscriber.receive().await;
            match request {
                CalculatorFunctions::Add(req) => {
                    let tokio_pub_sub::Request { content, response_sender } = req;
                    let response = self.add(content).await;
                    let _ = response_sender.send(response);
                }
                CalculatorFunctions::Multiply(req) => {
                    let tokio_pub_sub::Request { content, response_sender } = req;
                    let response = self.multiply(content).await;
                    let _ = response_sender.send(response);
                }
                CalculatorFunctions::Stringify(req) => {
                    let tokio_pub_sub::Request { content, response_sender } = req;
                    let response = self.stringify(content).await;
                    let _ = response_sender.send(response);
                }
            }
        }
    }
}
