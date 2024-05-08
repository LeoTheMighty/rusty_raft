use colored::{Color, Colorize};

use tonic::{transport::Server, Request, Response, Status};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::raft::color::pick_random_color;
use crate::raft::protobufs::{RaftMessage, RaftResponse};
use crate::raft::protobufs::raft_service_server::{RaftService, RaftServiceServer};
use crate::raft::protobufs::raft_service_client::RaftServiceClient;

#[derive(Default)]
pub struct RustyRaft {
    node_id: String,
    color: Color,
    server_address: String,
    client_addresses: Arc<Mutex<Vec<String>>>,
}

#[tonic::async_trait]
impl RaftService for RustyRaft {
    async fn send_raft_message(
        &self,
        request: Request<RaftMessage>,
    ) -> Result<Response<RaftResponse>, Status> {
        let message = request.into_inner().data;
        println!("Received: {}", message);

        let reply = RaftResponse {
            result: format!("Processed: {}", message),
        };
        Ok(Response::new(reply))
    }
}

impl RustyRaft {
    pub async fn new(node_id: String, server_address: String, client_addresses: Vec<String>) -> Self {
        let color = pick_random_color();
        RustyRaft {
            node_id,
            color,
            server_address,
            client_addresses: Arc::new(Mutex::new(client_addresses)),
        }
    }

    pub async fn run_server(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = self.server_address.clone().parse()?;
        println!("Server listening on {}", addr);

        Server::builder()
            .add_service(RaftServiceServer::new(self.clone()))
            .serve(addr)
            .await?;

        Ok(())
    }

    pub async fn send_message_to_clients(&self, data: String) -> Result<(), Box<dyn std::error::Error>> {
        let clients = self.client_addresses.lock().await;
        for address in clients.iter() {
            let mut client = RaftServiceClient::connect(address.clone()).await?;
            let request = Request::new(RaftMessage { data: data.clone() });

            match client.send_raft_message(request).await {
                Ok(response) => println!("Received response: {:?}", response.into_inner()),
                Err(e) => println!("Error sending message to {}: {}", address, e),
            }
        }
        Ok(())
    }

    pub fn log(&self, message: String) {
        println!("{}", format!("[{}]: {}", self.node_id, message).color(self.color));
    }
}

impl Clone for RustyRaft {
    fn clone(&self) -> Self {
        RustyRaft {
            node_id: self.node_id.clone(),
            color: self.color.clone(),
            server_address: self.server_address.clone(),
            client_addresses: Arc::clone(&self.client_addresses),
        }
    }
}