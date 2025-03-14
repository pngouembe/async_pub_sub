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

#[rpc_interface]
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
