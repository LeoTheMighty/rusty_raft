use colored::Colorize;

use std::net::SocketAddr;
use tonic::{transport::Server, Request, Response, Status};
use std::sync::Arc;
use tokio::sync::{Mutex, watch};
use tokio::task;

use crate::raft::color::{RandomColor, pick_random_color};
use crate::raft::protobufs::{Ack, Heartbeat, RaftMessage, RaftResponse};
use crate::raft::protobufs::raft_service_server::{RaftService, RaftServiceServer};
use crate::raft::protobufs::raft_service_client::RaftServiceClient;

struct Client {
    address: String,
    node_id: String,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Node {}: ({})]", self.node_id, self.address)
    }
}

#[derive(Default)]
pub struct RustyRaft {
    node_id: String,
    color: RandomColor,
    server_address: String,
    clients: Arc<Mutex<Vec<Client>>>,

    // Other fields
    shutdown_tx: Arc<Mutex<Option<watch::Sender<()>>>>,
}

#[tonic::async_trait]
impl RaftService for RustyRaft {
    async fn process_raft_message(
        &self,
        request: Request<RaftMessage>,
    ) -> Result<Response<RaftResponse>, Status> {
        let message = request.into_inner().data;
        self.log(format!("Received: {}", message));

        let reply = RaftResponse {
            result: format!("Processed: {}", message),
        };
        Ok(Response::new(reply))
    }

    async fn process_heartbeat(&self, request: Request<Heartbeat>) -> Result<Response<Ack>, Status> {
        let heartbeat = request.into_inner();

        Ok(Response::new(Ack {
            term: heartbeat.term,
            success: true,
        }))
    }
}

impl RustyRaft {
    pub async fn new(node_id: String, server_address: String, clients_info: Vec<(String, String)>) -> Self {
        let clients = clients_info.iter().map(|info| Client {
            address: info.0.clone(),
            node_id: info.1.clone(),
        }).collect();

        RustyRaft {
            node_id,
            color: RandomColor(pick_random_color()),
            server_address,
            clients: Arc::new(Mutex::new(clients)),
            shutdown_tx: Arc::new(Mutex::new(None)),
        }
    }

    async fn run_server(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = self.server_address.clone().parse()?;
        self.log(format!("Server listening on {}", addr));

        let mut shutdown_rx = self.shutdown_tx.lock().await.as_ref().unwrap().subscribe();

        let server = Server::builder()
            .add_service(RaftServiceServer::new(self.clone()))
            .serve_with_shutdown(addr, async {
                shutdown_rx.changed().await.ok();
            });

        self.log("Server started...".to_string());

        server.await?;

        self.log("Server shut down gracefully.".to_string());
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
        let clients = self.clients.lock().await;
        for client in clients.iter() {
            let address = client.address.clone();
            self.log(format!("Sending message to: {:?}", client));

            // Ensure the address has the http:// scheme
            let url = if address.starts_with("http://") || address.starts_with("https://") {
                address.clone()
            } else {
                format!("http://{}", address)
            };

            match RaftServiceClient::connect(url.clone()).await {
                Ok(mut client) => {
                    let request = Request::new(RaftMessage { data: data.clone() });

                    match client.process_raft_message(request).await {
                        Ok(response) => self.log(format!("Received response: {:?}", response.into_inner())),
                        Err(e) => eprintln!("Error sending message to {}: {}", address, e),
                    }
                }
                Err(e) => eprintln!("Error connecting to {}: {}", address, e),
            }
        }
        Ok(())
    }

    pub async fn send_heartbeats_to_clients(&self) -> Result<(), Box<dyn std::error::Error>> {
        let clients = self.clients.lock().await;
        for client in clients.iter() {
            let address = client.address.clone();
            self.log(format!("Sending heartbeat to: {:?}", client));

            // Ensure the address has the http:// scheme
            let url = if address.starts_with("http://") || address.starts_with("https://") {
                address.clone()
            } else {
                format!("http://{}", address)
            };

            match RaftServiceClient::connect(url.clone()).await {
                Ok(mut client) => {
                    let request = Request::new(Heartbeat {
                        term: 0,
                        leader_id: self.node_id.clone(),
                    });

                    match client.process_heartbeat(request).await {
                        Ok(response) => self.log(format!("Received Ack Response: {:?}", response.into_inner())),
                        Err(e) => eprintln!("Error sending heartbeat to {}: {}", address, e),
                    }
                }
                Err(e) => eprintln!("Error connecting to {}: {}", address, e),
            }

        }
        Ok(())
    }

    pub fn log(&self, message: String) {
        println!("{}", format!("[Node: {}]: {}", self.node_id, message).color(*self.color));
    }
}

impl Clone for RustyRaft {
    fn clone(&self) -> Self {
        RustyRaft {
            node_id: self.node_id.clone(),
            color: RandomColor((*self.color).clone()),
            server_address: self.server_address.clone(),
            clients: Arc::clone(&self.clients),
            shutdown_tx: Arc::clone(&self.shutdown_tx),
        }
    }
}