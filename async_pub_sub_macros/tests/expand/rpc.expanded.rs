use async_pub_sub_macros::rpc_interface;
pub trait RpcInterface {
    async fn add_one(&self, value: i32) -> i32;
    async fn add(&self, left: i32, right: i32) -> i32;
    async fn prefix_with_bar(&self, string: String) -> String;
    async fn get_toto(&self) -> String;
    async fn set_tata(&mut self, tata: String);
}
pub enum RpcInterfaceMessage {
    AddOne(async_pub_sub::Request<i32, i32>),
    Add(async_pub_sub::Request<(i32, i32), i32>),
    PrefixWithBar(async_pub_sub::Request<String, String>),
    GetToto(async_pub_sub::Request<(), String>),
    SetTata(async_pub_sub::Request<String, ()>),
}
#[automatically_derived]
impl ::core::fmt::Debug for RpcInterfaceMessage {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            RpcInterfaceMessage::AddOne(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "AddOne", &__self_0)
            }
            RpcInterfaceMessage::Add(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Add", &__self_0)
            }
            RpcInterfaceMessage::PrefixWithBar(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "PrefixWithBar",
                    &__self_0,
                )
            }
            RpcInterfaceMessage::GetToto(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "GetToto",
                    &__self_0,
                )
            }
            RpcInterfaceMessage::SetTata(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "SetTata",
                    &__self_0,
                )
            }
        }
    }
}
pub trait RpcInterfaceClient: async_pub_sub::MultiPublisher<RpcInterfaceMessage> {
    async fn add_one(&self, value: i32) -> i32 {
        let (request, response) = async_pub_sub::Request::new(value);
        self.publish(RpcInterfaceMessage::AddOne(request)).await.unwrap();
        response.await.unwrap()
    }
    async fn add(&self, left: i32, right: i32) -> i32 {
        let (request, response) = async_pub_sub::Request::new((left, right));
        self.publish(RpcInterfaceMessage::Add(request)).await.unwrap();
        response.await.unwrap()
    }
    async fn prefix_with_bar(&self, string: String) -> String {
        let (request, response) = async_pub_sub::Request::new(string);
        self.publish(RpcInterfaceMessage::PrefixWithBar(request)).await.unwrap();
        response.await.unwrap()
    }
    async fn get_toto(&self) -> String {
        let (request, response) = async_pub_sub::Request::new(());
        self.publish(RpcInterfaceMessage::GetToto(request)).await.unwrap();
        response.await.unwrap()
    }
    async fn set_tata(&mut self, tata: String) {
        let (request, response) = async_pub_sub::Request::new(tata);
        self.publish(RpcInterfaceMessage::SetTata(request)).await.unwrap();
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
pub trait RpcInterfaceServer: async_pub_sub::MultiSubscriber<
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
                let async_pub_sub::Request { content, response_sender } = req;
                let response = <Self as RpcInterface>::add_one(self, content).await;
                let _ = response_sender.send(response);
            }
            RpcInterfaceMessage::Add(req) => {
                let async_pub_sub::Request { content, response_sender } = req;
                let (left, right) = content;
                let response = <Self as RpcInterface>::add(self, left, right).await;
                let _ = response_sender.send(response);
            }
            RpcInterfaceMessage::PrefixWithBar(req) => {
                let async_pub_sub::Request { content, response_sender } = req;
                let response = <Self as RpcInterface>::prefix_with_bar(self, content)
                    .await;
                let _ = response_sender.send(response);
            }
            RpcInterfaceMessage::GetToto(req) => {
                let async_pub_sub::Request { content, response_sender } = req;
                let response = <Self as RpcInterface>::get_toto(self).await;
                let _ = response_sender.send(response);
            }
            RpcInterfaceMessage::SetTata(req) => {
                let async_pub_sub::Request { content, response_sender } = req;
                let response = <Self as RpcInterface>::set_tata(self, content).await;
                let _ = response_sender.send(response);
            }
        }
    }
}
impl<T> RpcInterfaceServer for T
where
    T: RpcInterface + async_pub_sub::MultiSubscriber<RpcInterfaceMessage>,
{}
