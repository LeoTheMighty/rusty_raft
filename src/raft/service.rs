use colored::Colorize;

use tonic::{transport::Server, Request, Response, Status};
use std::sync::Arc;
use tokio::sync::{Mutex, watch};
use tokio::task;

use crate::raft::color::{RandomColor, pick_random_color};
use crate::raft::protobufs::{RaftMessage, RaftResponse};
use crate::raft::protobufs::raft_service_server::{RaftService, RaftServiceServer};
use crate::raft::protobufs::raft_service_client::RaftServiceClient;

#[derive(Default)]
pub struct RustyRaft {
    node_id: String,
    color: RandomColor,
    server_address: String,
    client_addresses: Arc<Mutex<Vec<String>>>,

    // Other fields
    shutdown_tx: Arc<Mutex<Option<watch::Sender<()>>>>,
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
        RustyRaft {
            node_id,
            color: RandomColor(pick_random_color()),
            server_address,
            client_addresses: Arc::new(Mutex::new(client_addresses)),
            shutdown_tx: Arc::new(Mutex::new(None)),
        }
    }

    async fn run_server(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = self.server_address.clone().parse()?;
        println!("Server listening on {}", addr);

        let mut shutdown_rx = self.shutdown_tx.lock().await.as_ref().unwrap().subscribe();

        let server = Server::builder()
            .add_service(RaftServiceServer::new(self.clone()))
            .serve_with_shutdown(addr, async {
                shutdown_rx.changed().await.ok();
            });

        self.log("Server started...".to_string());

        server.await?;

        println!("Server shut down gracefully.");
        Ok(())
    }

    pub async fn start_server(&self) {
        let raft_clone = self.clone();
        let (shutdown_tx, _shutdown_rx) = watch::channel(());
        *self.shutdown_tx.lock().await = Some(shutdown_tx);

        task::spawn(async move {
            if let Err(e) = raft_clone.run_server().await {
                eprintln!("Server error: {}", e);
            }
        });
    }

    pub async fn stop_server(&self) {
        if let Some(shutdown_tx) = self.shutdown_tx.lock().await.take() {
            let _ = shutdown_tx.send(());
        }
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
        println!("{}", format!("[{}]: {}", self.node_id, message).color(*self.color));
    }
}

impl Clone for RustyRaft {
    fn clone(&self) -> Self {
        RustyRaft {
            node_id: self.node_id.clone(),
            color: RandomColor((*self.color).clone()),
            server_address: self.server_address.clone(),
            client_addresses: Arc::clone(&self.client_addresses),
            shutdown_tx: Arc::clone(&self.shutdown_tx),
        }
    }
}