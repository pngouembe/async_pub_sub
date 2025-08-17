#![allow(unused_imports)]
use async_pub_sub_macros::rpc_interface;

#[rpc_interface]
pub trait TestRpc {
    async fn add_one(&self, value: i32) -> i32;
}

// Test structure containing the client with default generic parameter
pub struct ServiceWithDefaultClient {
    pub client: TestRpcClient,
    pub _name: String,
}

// Test structure containing the client with explicit generic parameter  
pub struct ServiceWithGenericClient<T>
where
    T: Send + 'static,
{
    pub client: TestRpcClient<T>,
    pub _name: String,
}

// Test structure containing the client with specific type
pub struct ServiceWithStringClient {
    pub client: TestRpcClient<String>,
    pub _name: String,
}

fn main() {}