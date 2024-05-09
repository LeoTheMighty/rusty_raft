use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::raft::raft::RustyRaft;
use crate::raft::protobufs::{Ack, Heartbeat, RaftMessage, RaftResponse, RequestVote, RequestVoteResponse};
use crate::raft::protobufs::raft_service_server::RaftService;
use crate::raft::time::TimeoutHandler;
use crate::raft::state::Role;
use crate::raft::types::DynamicError;

#[tonic::async_trait]
impl RaftService for Arc<RustyRaft> {
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

        let mut state = self.state.lock().await;
        if state.current_term < heartbeat.term {
            state.current_term = heartbeat.term;
            state.role = Role::Follower;
            self.clone().reset_idle_timeout();
        }

        Ok(Response::new(Ack {
            term: state.current_term,
            success: true,
        }))
    }

    async fn process_request_vote(&self, request: Request<RequestVote>) -> Result<Response<RequestVoteResponse>, Status> {
        let request_vote = request.into_inner();

        let mut state = self.state.lock().await;
        let vote_granted: bool = if request_vote.term >= state.current_term {
            state.current_term = request_vote.term;

            if state.voted_for.is_none() {
                state.voted_for = Some(request_vote.candidate_id);

                true
            } else if state.voted_for == Some(request_vote.candidate_id) {
                true
            } else {
                false
            }
        } else {
            false
        };

        self.clone().reset_idle_timeout();

        Ok(Response::new(RequestVoteResponse {
            vote_granted,
            term: state.current_term,
        }))
    }
}

impl RustyRaft {
    pub async fn send_heartbeats_to_clients(self: Arc<Self>) -> Result<(), DynamicError> {
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

    pub async fn send_request_votes_to_clients(self: Arc<Self>) -> Result<(), DynamicError> {
        for client in self.clients.iter() {
            self.log(format!("Sending request vote to: {:?}", client));
            let request = RequestVote { term: 0, candidate_id: self.node_id.clone() };
            match client.send_request_vote(request).await {
                Ok(response) => {
                    let mut state = self.state.lock().await;
                    if state.role == Role::Follower {
                        self.clone().set_election_timeout();

                        if response.vote_granted {
                            state.votes_received += 1;

                            if state.votes_received > (self.clients.len() / 2) as u32 {
                                state.role = Role::Leader;

                                match self.clone().send_heartbeats_to_clients().await {
                                    Ok(_) => (),
                                    Err(e) => self.log(format!("Error sending heartbeats: {}", e)),
                                }

                                return Ok(());
                            }
                        }
                    }
                },
                Err(e) => eprintln!("Error sending request vote to {:?}: {}", client, e),
            }
        }

        Ok(())
    }

    pub fn set_heartbeat_timeout(self: Arc<Self>) {
        let timeout_handler = Arc::clone(&self.timeout_handler);
        let timeout = TimeoutHandler::get_heartbeat_timeout();
        tokio::spawn(async move {
            timeout_handler.set_timeout(timeout, async move {
                self.handle_timeout().await;
            }).await
        });
    }

    pub fn set_election_timeout(self: Arc<Self>) {
        let timeout_handler = Arc::clone(&self.timeout_handler);
        let timeout = TimeoutHandler::get_election_timeout();
        tokio::spawn(async move {
            timeout_handler.set_timeout(timeout, async move {
                self.handle_timeout().await;
            }).await
        });
    }

    pub fn reset_idle_timeout(self: Arc<Self>) {
        let timeout_handler = Arc::clone(&self.timeout_handler);
        let timeout = TimeoutHandler::get_random_timeout();
        tokio::spawn(async move {
            timeout_handler.set_timeout(timeout, async move {
                self.handle_timeout().await;
            }).await
        });
    }

    pub fn cancel_timeout(self: Arc<Self>) {
        let timeout_handler = Arc::clone(&self.timeout_handler);
        tokio::spawn(async move {
            timeout_handler.cancel_timeout().await;
        });
    }

    pub async fn handle_timeout(self: Arc<Self>) {
        let mut state = self.state.lock().await;

        match state.role {
            Role::Follower | Role::Candidate => {
                state.votes_received = 1;
                state.current_term += 1;

                self.clone().set_election_timeout();

                match self.clone().send_request_votes_to_clients().await {
                    Ok(_) => (),
                    Err(e) => self.log(format!("Error sending request votes: {}", e)),
                }
            }
            Role::Leader => {
                match self.clone().send_heartbeats_to_clients().await {
                    Ok(_) => (),
                    Err(e) => self.log(format!("Error sending heartbeats: {}", e)),
                }
            }
        }
    }

    // pub async fn set_leader_role(self: Arc<Self>) {
    //
    // }
    //
    // pub async fn set_follower_role(self: Arc<Self>) {
    //
    // }
}