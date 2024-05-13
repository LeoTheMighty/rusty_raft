use tonic::Request;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::raft::protobufs::{Ack, AppendEntries, AppendEntriesResponse, Heartbeat, Redirect, RedirectResponse, RequestVote, RequestVoteResponse};
use crate::raft::protobufs::raft_service_client::RaftServiceClient;
use crate::raft::types::DynamicError;

#[derive(Clone, Default)]
pub struct ClientState {
    // index of next log entry to send to server
    pub next_index: u64,
    // index of highest log entry known to be replicated on server
    pub match_index: u64,
}

impl ClientState {
    pub fn new() -> Self {
        Self {
            next_index: 0,
            match_index: 0,
        }
    }
}

pub struct Client {
    pub address: String,
    url: String,
    pub node_id: String,
    pub state: Arc<Mutex<ClientState>>,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Node {}: ({})]", self.node_id, self.address)
    }
}

impl Clone for Client {
    fn clone(&self) -> Self {
        Self {
            address: self.address.clone(),
            url: self.url.clone(),
            node_id: self.node_id.clone(),
            state: Arc::clone(&self.state),
        }
    }
}

impl Client {
    pub fn new(address: String, node_id: String) -> Self {
        // Ensure the url has the http:// scheme
        let url = if address.starts_with("http://") || address.starts_with("https://") {
            address.clone()
        } else {
            format!("http://{}", address)
        };

        Self {
            address,
            url,
            node_id,
            state: Arc::new(Mutex::new(ClientState::new())),
        }
    }

    pub async fn send_heartbeat(&self, request: Heartbeat) -> Result<Ack, DynamicError> {
        let url = self.url.clone();
        match RaftServiceClient::connect(url.clone()).await {
            Ok(mut client) => {
                match client.process_heartbeat(Request::new(request)).await {
                    Ok(response) => Ok(response.into_inner()),
                    Err(e) => Err(Box::new(e) as DynamicError),
                }
            }
            Err(e) => Err(Box::new(e) as DynamicError),
        }
    }

    pub async fn send_request_vote(&self, request: RequestVote) -> Result<RequestVoteResponse, DynamicError> {
        let url = self.url.clone();
        match RaftServiceClient::connect(url.clone()).await {
            Ok(mut client) => {
                match client.process_request_vote(Request::new(request)).await {
                    Ok(response) => Ok(response.into_inner()),
                    Err(e) => Err(Box::new(e) as DynamicError),
                }
            }
            Err(e) => Err(Box::new(e) as DynamicError),
        }
    }

    pub async fn send_append_entries(&self, request: AppendEntries) -> Result<AppendEntriesResponse, DynamicError> {
        let url = self.url.clone();
        match RaftServiceClient::connect(url.clone()).await {
            Ok(mut client) => {
                match client.process_append_entries(Request::new(request)).await {
                    Ok(response) => Ok(response.into_inner()),
                    Err(e) => Err(Box::new(e) as DynamicError),
                }
            }
            Err(e) => Err(Box::new(e) as DynamicError),
        }
    }

    pub async fn send_redirect(&self, message: String) -> Result<RedirectResponse, DynamicError> {
        let url = self.url.clone();
        match RaftServiceClient::connect(url.clone()).await {
            Ok(mut client) => {
                match client.process_redirect(Request::new(Redirect { data: message })).await {
                    Ok(response) => Ok(response.into_inner()),
                    Err(e) => Err(Box::new(e) as DynamicError),
                }
            }
            Err(e) => Err(Box::new(e) as DynamicError),
        }
    }
}
