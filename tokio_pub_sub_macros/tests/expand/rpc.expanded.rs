use tokio_pub_sub_macros::rpc_interface;
pub trait RpcInterface {
    async fn add_one(&self, value: i32) -> i32;
    async fn add(&self, left: i32, right: i32) -> i32;
    async fn prefix_with_bar(&self, string: String) -> String;
    async fn get_toto(&self) -> String;
    async fn set_tata(&mut self, tata: String);
}
pub enum RpcInterfaceMessage {
    AddOne(tokio_pub_sub::Request<i32, i32>),
    Add(tokio_pub_sub::Request<(i32, i32), i32>),
    PrefixWithBar(tokio_pub_sub::Request<String, String>),
    GetToto(tokio_pub_sub::Request<(), String>),
    SetTata(tokio_pub_sub::Request<String, ()>),
}
pub trait RpcInterfaceClient: tokio_pub_sub::MultiPublisher<RpcInterfaceMessage> {
    async fn add_one(&self, value: i32) -> i32 {
        let (request, response) = tokio_pub_sub::Request::new(value);
        self.publish_event(RpcInterfaceMessage::AddOne(request)).await.unwrap();
        response.await.unwrap()
    }
    async fn add(&self, left: i32, right: i32) -> i32 {
        let (request, response) = tokio_pub_sub::Request::new((left, right));
        self.publish_event(RpcInterfaceMessage::Add(request)).await.unwrap();
        response.await.unwrap()
    }
    async fn prefix_with_bar(&self, string: String) -> String {
        let (request, response) = tokio_pub_sub::Request::new(string);
        self.publish_event(RpcInterfaceMessage::PrefixWithBar(request)).await.unwrap();
        response.await.unwrap()
    }
    async fn get_toto(&self) -> String {
        let (request, response) = tokio_pub_sub::Request::new(());
        self.publish_event(RpcInterfaceMessage::GetToto(request)).await.unwrap();
        response.await.unwrap()
    }
    async fn set_tata(&mut self, tata: String) {
        let (request, response) = tokio_pub_sub::Request::new(tata);
        self.publish_event(RpcInterfaceMessage::SetTata(request)).await.unwrap();
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
    async fn add(&self, left: i32, right: i32) -> i32 {
        <Self as RpcInterfaceClient>::add(self, left, right).await
    }
    async fn prefix_with_bar(&self, string: String) -> String {
        <Self as RpcInterfaceClient>::prefix_with_bar(self, string).await
    }
    async fn get_toto(&self) -> String {
        <Self as RpcInterfaceClient>::get_toto(self).await
    }
    async fn set_tata(&mut self, tata: String) {
        <Self as RpcInterfaceClient>::set_tata(self, tata).await
    }
}
pub trait RpcInterfaceServer: tokio_pub_sub::MultiSubscriber<
        RpcInterfaceMessage,
    > + RpcInterface {
    async fn run(&mut self) {
        loop {
            let request = self.receive().await;
            self.handle_request(request).await;
        }
    }
    async fn handle_request(&mut self, request: RpcInterfaceMessage) {
        match request {
            RpcInterfaceMessage::AddOne(req) => {
                let tokio_pub_sub::Request { content, response_sender } = req;
                let response = <Self as RpcInterface>::add_one(self, content).await;
                let _ = response_sender.send(response);
            }
            RpcInterfaceMessage::Add(req) => {
                let tokio_pub_sub::Request { content, response_sender } = req;
                let (left, right) = content;
                let response = <Self as RpcInterface>::add(self, left, right).await;
                let _ = response_sender.send(response);
            }
            RpcInterfaceMessage::PrefixWithBar(req) => {
                let tokio_pub_sub::Request { content, response_sender } = req;
                let response = <Self as RpcInterface>::prefix_with_bar(self, content)
                    .await;
                let _ = response_sender.send(response);
            }
            RpcInterfaceMessage::GetToto(req) => {
                let tokio_pub_sub::Request { content, response_sender } = req;
                let response = <Self as RpcInterface>::get_toto(self).await;
                let _ = response_sender.send(response);
            }
            RpcInterfaceMessage::SetTata(req) => {
                let tokio_pub_sub::Request { content, response_sender } = req;
                let response = <Self as RpcInterface>::set_tata(self, content).await;
                let _ = response_sender.send(response);
            }
        }
    }
}
impl<T> RpcInterfaceServer for T
where
    T: RpcInterface + tokio_pub_sub::MultiSubscriber<RpcInterfaceMessage>,
{}
