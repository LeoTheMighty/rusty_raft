use std::sync::Arc;
use tonic::{Request, Response, Status};
use tokio::task;

use crate::raft::client::Client;
use crate::raft::log::Entry;
use crate::raft::rusty_raft::RustyRaft;
use crate::raft::protobufs::{Ack, AppendEntries, AppendEntriesResponse, Heartbeat, RaftMessage, RaftResponse, Redirect, RedirectResponse, RequestVote, RequestVoteResponse};
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

        let (mut current_term, role) = {
            let state = self.state.lock().await;

            (state.current_term, state.role.clone())
        };

        if role == Role::Candidate || current_term < heartbeat.term {
            {
                let mut state = self.state.lock().await;
                state.current_term = heartbeat.term;
                current_term = heartbeat.term;
            }

            self.clone().set_follower_role().await;
        }

        self.clone().reset_idle_timeout();

        Ok(Response::new(Ack {
            term: current_term,
            success: true,
        }))
    }

    async fn process_append_entries(&self, request: Request<AppendEntries>) -> Result<Response<AppendEntriesResponse>, Status> {
        let append_entries = request.into_inner();

        self.log(format!("Received append entries: {:?}", append_entries));

        self.clone().reset_idle_timeout();

        let (mut current_term, role) = {
            let state = self.state.lock().await;

            (state.current_term, state.role.clone())
        };

        if append_entries.term < current_term {
            return Ok(Response::new(AppendEntriesResponse {
                term: current_term,
                success: false,
            }));
        }

        let prev_entry_term = {
            self.log.lock().await.entries[append_entries.prev_log_index as usize].term
        };

        if prev_entry_term != append_entries.prev_log_term {
            return Ok(Response::new(AppendEntriesResponse {
                term: current_term,
                success: false,
            }));
        }

        if role == Role::Candidate || current_term < append_entries.term {
            {
                let mut state = self.state.lock().await;
                state.current_term = append_entries.term;
                current_term = append_entries.term;
            }

            self.clone().set_follower_role().await;
        }

        self.log.lock().await.append_entries(
            append_entries.prev_log_index + 1,
            append_entries.entries
        );

        Ok(Response::new(AppendEntriesResponse {
            term: current_term,
            success: true,
        }))
    }

    async fn process_redirect(&self, request: Request<Redirect>) -> Result<Response<RedirectResponse>, Status> {
        let redirect = request.into_inner();

        self.log(format!("Received redirect: {:?}", redirect));

        let role = {
            self.state.lock().await.role.clone()
        };

        if role == Role::Leader {
            let self_clone = self.clone();
            tokio::spawn(async move {
                match self_clone.handle_request(redirect.data).await {
                    Ok(_) => (),
                    Err(err) => eprintln!("Error handling redirect: {}", err),
                }
            });

            return Ok(Response::new(RedirectResponse { success: true }));
        }

        Ok(Response::new(RedirectResponse { success: false }))
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
            } else {
                state.voted_for == Some(request_vote.candidate_id)
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
    pub async fn handle_request(self: Arc<Self>, message: String) -> Result<(), DynamicError> {
        self.log(format!("Received Client Message: {}", message));

        let role = {
            self.state.lock().await.role.clone()
        };

        if role == Role::Leader {
            // Add entry to log, update state, and send append entries to all followers
            {
                let mut log = self.log.lock().await;
                let state = self.state.lock().await;

                log.append_entry(Entry { term: state.current_term, message });
            }

            match self.clone().send_append_entries_to_all_clients().await {
                Ok(_) => (),
                Err(e) => eprintln!("Error sending append entries to all clients: {}", e),
            }
        } else {
            // Redirect request to all clients
            match self.clone().send_redirect_to_all_clients(message).await {
                Ok(_) => (),
                Err(e) => eprintln!("Error sending redirect to all clients: {}", e),
            }
        }

        Ok(())
    }

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
        self.log("Sending heartbeats to all clients".to_string());

        match self.run_for_all_clients(|self_clone, client| async move {
            self_clone.log(format!("Sending heartbeat to: {:?}", client));
            let term = {
                self_clone.state.lock().await.current_term
            };
            let request = Heartbeat { term, leader_id: self_clone.node_id.clone() };
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

    pub async fn send_append_entries_to_all_clients(self: Arc<Self>) -> Result<(), DynamicError> {
        self.log("Sending append entries to all clients".to_string());

        match self.run_for_all_clients(|self_clone, client| async move {
            self_clone.log(format!("Sending append entries to: {:?}", client));
            let request = {
                let state = self_clone.state.lock().await;
                let client_state = client.state.lock().await;
                let log = self_clone.log.lock().await;

                let prev_log_index = client_state.next_index - 1;
                let prev_log_term = log.entries[prev_log_index as usize].term;
                let entries = log.get_log_entries(client_state.next_index);

                AppendEntries {
                    term: state.current_term,
                    leader_id: self_clone.node_id.clone(),
                    prev_log_index,
                    prev_log_term,
                    leader_commit: log.commit_index,
                    entries,
                }
            };

            match client.send_append_entries(request).await {
                Ok(response) => self_clone.handle_append_entries_response(response).await,
                Err(e) => eprintln!("Error sending append entries to {:?}: {}", client, e),
            }
        }).await {
            Ok(_) => (),
            Err(e) => return Err(e),
        };

        Ok(())
    }

    pub async fn handle_append_entries_response(self: Arc<Self>, response: AppendEntriesResponse) {
        self.log(format!("Received Append Entries Response: {:?}", response));
    }

    pub async fn send_redirect_to_all_clients(self: Arc<Self>, message: String) -> Result<(), DynamicError> {
        self.log("Sending redirect to all clients".to_string());

        let message_clone = Arc::new(message.clone());
        let result = self.run_for_all_clients(move |self_clone, client| {
            let message_clone = Arc::clone(&message_clone);
            async move {
                self_clone.log(format!("Sending redirect to: {:?}", client));
                let message_clone = Arc::clone(&message_clone);
                match client.send_redirect(message_clone.to_string()).await {
                    Ok(response) => self_clone.handle_redirect_response(response).await,
                    Err(e) => eprintln!("Error sending redirect to {:?}: {}", client, e),
                }
            }
        }).await;

        match result {
            Ok(_) => (),
            Err(e) => return Err(e),
        }

        Ok(())
    }

    pub async fn handle_redirect_response(self: Arc<Self>, response: RedirectResponse) {
        self.log(format!("Received Redirect Response: {:?}", response));
    }

    pub async fn send_request_votes_to_clients(self: Arc<Self>) -> Result<(), DynamicError> {
        self.log("Sending request votes to all clients".to_string());

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
        self.log(format!("Received Request Vote Response: {:?}", response));

        let role = {
            let state = self.state.lock().await;

            state.role.clone()
        };

        if role == Role::Candidate {
            self.clone().set_election_timeout();

            if response.vote_granted {
                let votes_received = {
                    let mut state = self.state.lock().await;
                    state.votes_received += 1;

                    state.votes_received
                };

                if votes_received > (self.clients.len() / 2) as u32 {
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
        {
            let mut state = self.state.lock().await;
            if state.role != Role::Leader {
                state.role = Role::Leader;

                self.log("Starting Leader Role".to_string());
            }
        }

        self.clone().set_heartbeat_timeout();

        match self.clone().send_heartbeats_to_clients().await {
            Ok(_) => (),
            Err(e) => self.log(format!("Error sending heartbeats: {}", e)),
        }
    }

    pub async fn set_candidate_role(self: Arc<Self>) {
        {
            let mut state = self.state.lock().await;
            if state.role != Role::Candidate {
                self.log("Starting Election in Candidate Role".to_string());

                state.role = Role::Candidate;
            }

            state.votes_received = 1;
            state.current_term += 1;
            state.voted_for = Some(self.node_id.clone());
        }

        self.clone().set_election_timeout();

        match self.clone().send_request_votes_to_clients().await {
            Ok(_) => (),
            Err(e) => self.log(format!("Error sending request votes: {}", e)),
        }
    }

    pub async fn set_follower_role(self: Arc<Self>) {
        self.log("Starting Follower Role".to_string());

        {
            let mut state = self.state.lock().await;
            state.role = Role::Follower;
            state.voted_for = None;
        }
    }
}