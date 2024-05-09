use std::sync::Arc;
use tonic::{Request, Response, Status};
use tokio::task;

use crate::raft::client::Client;
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

        self.log(format!("Received heartbeat: {:?}", heartbeat));

        let state = self.state.lock().await;
        if state.current_term < heartbeat.term {
            self.clone().set_follower_role().await;
        }

        Ok(Response::new(Ack {
            term: state.current_term,
            success: true,
        }))
    }

    async fn process_request_vote(&self, request: Request<RequestVote>) -> Result<Response<RequestVoteResponse>, Status> {
        let request_vote = request.into_inner();

        self.log(format!("Received request vote: {:?}", request_vote));

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
    async fn run_for_all_clients<F, Fut>(self: Arc<Self>, f: F) -> Result<(), DynamicError>
        where
            F: Fn(Arc<Self>, Client) -> Fut + Send + Sync + 'static,
            Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let mut tasks = vec![];

        for client in self.clients.iter().cloned() {
            let self_clone = self.clone();
            let task = task::spawn(f(self_clone, client));
            tasks.push(task);
        }

        for task in tasks {
            task.await?;
        }

        Ok(())
    }

    pub async fn send_heartbeats_to_clients(self: Arc<Self>) -> Result<(), DynamicError> {
        match self.run_for_all_clients(|self_clone, client| async move {
            self_clone.log(format!("Sending heartbeat to: {:?}", client));
            let request = Heartbeat { term: 0, leader_id: self_clone.node_id.clone() };
            match client.send_heartbeat(request).await {
                Ok(response) => self_clone.handle_heartbeat_response(response).await,
                Err(e) => eprintln!("Error sending heartbeat to {:?}: {}", client, e),
            }
        }).await {
            Ok(_) => (),
            Err(e) => return Err(e),
        }

        Ok(())
    }

    pub async fn handle_heartbeat_response(self: Arc<Self>, response: Ack) {
        self.log(format!("Received Ack Response: {:?}", response));
    }

    pub async fn send_request_votes_to_clients(self: Arc<Self>) -> Result<(), DynamicError> {
        match self.run_for_all_clients(|self_clone, client| async move {
            self_clone.log(format!("Sending request vote to: {:?}", client));
            let term = self_clone.state.lock().await.current_term;
            let request = RequestVote { term, candidate_id: self_clone.node_id.clone() };
            match client.send_request_vote(request).await {
                Ok(response) => self_clone.handle_request_vote_response(response).await,
                Err(e) => eprintln!("Error sending request vote to {:?}: {}", client, e),
            }
        }).await {
            Ok(_) => (),
            Err(e) => return Err(e),
        }

        Ok(())
    }

    pub async fn handle_request_vote_response(self: Arc<Self>, response: RequestVoteResponse) {
        let mut state = self.state.lock().await;
        if state.role == Role::Follower {
            self.clone().set_election_timeout();

            if response.vote_granted {
                state.votes_received += 1;

                if state.votes_received > (self.clients.len() / 2) as u32 {
                    self.clone().set_leader_role().await;
                }
            }
        }
    }

    pub fn set_heartbeat_timeout(self: Arc<Self>) {
        let timeout_handler = Arc::clone(&self.timeout_handler);
        let timeout = TimeoutHandler::get_heartbeat_timeout();
        tokio::spawn(async move {
            timeout_handler.set_timeout(timeout, async move {
                self.log("Heartbeat Timeout".to_string());
                self.handle_timeout().await;
            }).await
        });
    }

    pub fn set_election_timeout(self: Arc<Self>) {
        let timeout_handler = Arc::clone(&self.timeout_handler);
        let timeout = TimeoutHandler::get_election_timeout();
        tokio::spawn(async move {
            timeout_handler.set_timeout(timeout, async move {
                self.log("Election Timeout".to_string());
                self.handle_timeout().await;
            }).await
        });
    }

    pub fn reset_idle_timeout(self: Arc<Self>) {
        let timeout_handler = Arc::clone(&self.timeout_handler);
        let timeout = TimeoutHandler::get_random_timeout();
        tokio::spawn(async move {
            timeout_handler.set_timeout(timeout, async move {
                self.log("Idle Timeout".to_string());
                self.handle_timeout().await;
            }).await
        });
    }

    // pub fn cancel_timeout(self: Arc<Self>) {
    //     let timeout_handler = Arc::clone(&self.timeout_handler);
    //     tokio::spawn(async move {
    //         timeout_handler.cancel_timeout().await;
    //     });
    // }

    pub async fn handle_timeout(self: Arc<Self>) {
        let role = self.state.lock().await.role.clone();
        match role {
            Role::Follower | Role::Candidate => {
                self.clone().set_candidate_role().await;
            }
            Role::Leader => {
                self.clone().set_leader_role().await;
            }
        }
    }

    pub async fn set_leader_role(self: Arc<Self>) {
        self.log("Starting Leader Role".to_string());

        if let mut state = self.state.lock().await {
            state.role = Role::Leader;
        }

        self.clone().set_heartbeat_timeout();

        match self.clone().send_heartbeats_to_clients().await {
            Ok(_) => (),
            Err(e) => self.log(format!("Error sending heartbeats: {}", e)),
        }
    }

    pub async fn set_candidate_role(self: Arc<Self>) {
        self.log("Starting Candidate Role".to_string());

        if let mut state = self.state.lock().await {
            state.role = Role::Candidate;
            state.votes_received = 1;
            state.current_term += 1;
        }

        self.clone().set_election_timeout();

        match self.clone().send_request_votes_to_clients().await {
            Ok(_) => (),
            Err(e) => self.log(format!("Error sending request votes: {}", e)),
        }
    }

    pub async fn set_follower_role(self: Arc<Self>) {
        self.log("Starting Follower Role".to_string());

        if let mut state = self.state.lock().await {
            state.role = Role::Follower;
            state.voted_for = None;
            state.current_term += 1;
        }

        self.clone().reset_idle_timeout();
    }
}