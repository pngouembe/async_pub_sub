use tokio_pub_sub_macros::rpc_interface;
pub trait RpcInterface {
    async fn add_one(&self, value: i32) -> i32;
    async fn prefix_with_bar(&self, string: String) -> String;
}
pub enum RpcInterfaceMessage {
    AddOne(tokio_pub_sub::Request<i32, i32>),
    PrefixWithBar(tokio_pub_sub::Request<String, String>),
}
pub trait RpcInterfaceClient: tokio_pub_sub::Publisher<Message = RpcInterfaceMessage> {
    async fn add_one(&self, value: i32) -> i32 {
        let (request, response) = tokio_pub_sub::Request::new(value);
        self.publish_event(RpcInterfaceMessage::AddOne(request)).await.unwrap();
        response.await.unwrap()
    }
    async fn prefix_with_bar(&self, string: String) -> String {
        let (request, response) = tokio_pub_sub::Request::new(string);
        self.publish_event(RpcInterfaceMessage::PrefixWithBar(request)).await.unwrap();
        response.await.unwrap()
    }
}
impl<T> RpcInterface for T
where
    T: RpcInterfaceClient,
{
    async fn add_one(&self, value: i32) -> i32 {
        <Self as RpcInterfaceClient>::add_one(self, value).await
    }
    async fn prefix_with_bar(&self, string: String) -> String {
        <Self as RpcInterfaceClient>::prefix_with_bar(self, string).await
    }
}
pub trait RpcInterfaceServer: tokio_pub_sub::Subscriber<
        Message = RpcInterfaceMessage,
    > + RpcInterface {
    async fn run(&mut self) {
        loop {
            match self.receive().await {
                RpcInterfaceMessage::AddOne(req) => {
                    let tokio_pub_sub::Request { content, response_sender } = req;
                    let response = <Self as RpcInterface>::add_one(&self, content).await;
                    let _ = response_sender.send(response);
                }
                RpcInterfaceMessage::PrefixWithBar(req) => {
                    let tokio_pub_sub::Request { content, response_sender } = req;
                    let response = <Self as RpcInterface>::prefix_with_bar(
                            &self,
                            content,
                        )
                        .await;
                    let _ = response_sender.send(response);
                }
            }
        }
    }
}
impl<T> RpcInterfaceServer for T
where
    T: RpcInterface + tokio_pub_sub::Subscriber<Message = RpcInterfaceMessage>,
{}
