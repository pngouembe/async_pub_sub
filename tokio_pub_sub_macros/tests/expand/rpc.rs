use tokio_pub_sub_macros::rpc_interface;

#[rpc_interface]
pub trait RpcInterface {
    async fn add_one(&self, value: i32) -> i32;
    async fn prefix_with_bar(&self, string: String) -> String;
}
