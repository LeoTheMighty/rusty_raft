use tonic::Request;
use crate::raft::protobufs::{Ack, Heartbeat, RequestVote, RequestVoteResponse};
use crate::raft::protobufs::raft_service_client::RaftServiceClient;
use crate::raft::types::DynamicError;

pub struct Client {
    pub address: String,
    url: String,
    pub node_id: String,
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
            node_id
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
}
