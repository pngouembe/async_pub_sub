use async_pub_sub_macros::rpc_interface;

#[rpc_interface(Clone, Debug)]
trait TestRpc {
    async fn method_with_ref_return(&self) -> &String;
    async fn another_method(&self, arg1: u32) -> u32;
}

fn main() {}