use std::sync::Arc;
use tonic::{Request, Response, Status};
use tokio::sync::Mutex;

use crate::raft::raft::RustyRaft;
use crate::raft::protobufs::{Ack, Heartbeat, RaftMessage, RaftResponse};
use crate::raft::protobufs::raft_service_server::RaftService;
use crate::raft::state::State;

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

        self.reset_timeout();

        Ok(Response::new(Ack {
            term: heartbeat.term,
            success: true,
        }))
    }
}

impl RustyRaft {
    pub async fn send_heartbeats_to_clients(&self) -> Result<(), Box<dyn std::error::Error>> {
        for client in self.clients.iter() {
            self.log(format!("Sending heartbeat to: {:?}", client));
            let request = Heartbeat { term: 0, leader_id: self.node_id.clone() };
            match client.send_heartbeat(request).await {
                Ok(response) => self.log(format!("Received Ack Response: {:?}", response)),
                Err(e) => eprintln!("Error sending heartbeat to {:?}: {}", client, e),
            }
        }

        Ok(())
    }

    pub fn reset_timeout(&self) {
        let state = Arc::clone(&self.state);
        let timeout_handler = Arc::clone(&self.timeout_handler);
        tokio::spawn(async move {
            timeout_handler.set_random_timeout(async move {
                RustyRaft::handle_timeout(state).await;
            }).await
        });
    }

    pub fn cancel_timeout(&self) {
        let timeout_handler = Arc::clone(&self.timeout_handler);
        tokio::spawn(async move {
            timeout_handler.cancel_timeout().await;
        });
    }

    pub async fn handle_timeout(state: Arc<Mutex<State>>) {
        let state = state.lock().await;
        println!("{}: Timeout", state.current_term);
    }
}