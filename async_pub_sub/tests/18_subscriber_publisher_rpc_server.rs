// TODO: find a way to make Subscriber trait dyn compatible again

// use async_pub_sub::macros::{DerivePublisher, DeriveSubscriber, routes, rpc_interface};
// use async_pub_sub::{Publisher, PublisherImpl, Result, Subscriber, SubscriberImpl};

// #[rpc_interface]
// trait MyServerInterface {
//     async fn perform_action(&self, action: String) -> String;
//     async fn get_data(&self) -> String;
//     async fn set_data(&mut self, data: String);
//     async fn broadcast_stored_data(&self);
// }

// enum MyServerNotification {
//     DataUpdated(String),
// }

// #[derive(DeriveSubscriber, DerivePublisher)]
// struct MyServer {
//     data: String,

//     #[subscriber(MyServerInterfaceMessage)]
//     rpc_subscriber: Box<dyn Subscriber<InputMessage = MyServerInterfaceMessage> + Send + Sync>,

//     #[subscriber(MyServerNotification)]
//     notification_subscriber: Box<dyn Subscriber<InputMessage = MyServerNotification> + Send + Sync>,

//     #[publisher(String)]
//     data_publisher: Box<dyn Publisher<InputMessage = String> + Send + Sync>,
// }

// impl MyServerInterface for MyServer {
//     async fn perform_action(&self, action: String) -> String {
//         format!("Action performed: {action}")
//     }

//     async fn get_data(&self) -> String {
//         self.data.clone()
//     }

//     async fn set_data(&mut self, data: String) {
//         self.data = data;
//     }

//     async fn broadcast_stored_data(&self) {
//         self.data_publisher
//             .publish(self.data.clone())
//             .await
//             .unwrap();
//     }
// }

// impl MyServer {
//     async fn handle_rpc_request(&mut self, mut request: MyServerInterfaceMessage) {
//         use async_pub_sub::Request;
//         let content = request.take_content().expect("failed to get content");
//         match content {
//             MyServerInterfaceContent::PerformAction(action) => {
//                 let response = self.perform_action(action).await;
//                 let response_content = MyServerInterfaceResponse::PerformAction(response);
//                 request
//                     .respond(response_content)
//                     .await
//                     .expect("failed to send response");
//             }
//             MyServerInterfaceContent::GetData(()) => {
//                 let response = self.get_data().await;
//                 let response_content = MyServerInterfaceResponse::GetData(response);
//                 request
//                     .respond(response_content)
//                     .await
//                     .expect("failed to send response");
//             }
//             MyServerInterfaceContent::SetData(data) => {
//                 self.set_data(data).await;
//                 let response_content = MyServerInterfaceResponse::SetData(());
//                 request
//                     .respond(response_content)
//                     .await
//                     .expect("failed to send response");
//             }
//             MyServerInterfaceContent::BroadcastStoredData(()) => {
//                 self.broadcast_stored_data().await;
//                 let response_content = MyServerInterfaceResponse::BroadcastStoredData(());
//                 request
//                     .respond(response_content)
//                     .await
//                     .expect("failed to send response");
//             }
//         }
//     }

//     async fn run(&mut self) {
//         loop {
//             tokio::select! {
//                 MyServerNotification::DataUpdated(data) = self.notification_subscriber.receive() => {
//                     self.data = data;
//                 }
//                 request = self.rpc_subscriber.receive() => {
//                     self.handle_rpc_request(request).await;
//                 }
//             }
//         }
//     }
// }

// #[tokio::test]
// async fn test_pub_sub_rpc_server() -> Result<()> {
//     let mut server = MyServer {
//         data: String::from("Initial data"),
//         rpc_subscriber: Box::new(SubscriberImpl::new("rpc_subscriber")),
//         notification_subscriber: Box::new(SubscriberImpl::new("notification_subscriber")),
//         data_publisher: Box::new(PublisherImpl::new("data_publisher", 10)),
//     };

//     let mut rpc_client = MyServerInterfaceClient::new(PublisherImpl::new("client", 1));
//     let mut notification_publisher = PublisherImpl::new("notification_publisher", 1);
//     let mut data_subscriber = SubscriberImpl::new("data_subscriber");

//     routes! {
//         server -> data_subscriber,
//         rpc_client -> server,
//         notification_publisher -> server,
//     }?;

//     tokio::spawn(async move {
//         server.run().await;
//     });

//     tokio::spawn(async move {
//         let action_result = rpc_client.perform_action("Test action".to_string()).await;
//         assert_eq!(action_result, "Action performed: Test action");

//         let data_result = rpc_client.get_data().await;
//         assert_eq!(data_result, "Initial data");

//         rpc_client.set_data("Updated data".to_string()).await;
//         let updated_data = rpc_client.get_data().await;
//         assert_eq!(updated_data, "Updated data");

//         notification_publisher
//             .publish(MyServerNotification::DataUpdated("New data".to_string()))
//             .await
//             .unwrap();

//         rpc_client.broadcast_stored_data().await;

//         let notification = data_subscriber.receive().await;
//         assert_eq!(notification, "New data");
//     });

//     Ok(())
// }
