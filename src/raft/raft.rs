use std::sync::Arc;
use colored::Colorize;
use tokio::sync::{Mutex, watch};
use crate::raft::client::Client;
use crate::raft::color::{pick_random_color, RandomColor};
use crate::raft::time::TimeoutHandler;
use crate::raft::state::State;

#[derive(Default)]
pub struct RustyRaft {
    // Immutable State
    pub node_id: String,
    color: RandomColor,
    pub server_address: String,
    pub clients: Vec<Client>,

    // Mutable State
    pub state: Arc<Mutex<State>>,

    pub shutdown_tx: Arc<Mutex<Option<watch::Sender<()>>>>,
    pub timeout_handler: Arc<TimeoutHandler>,
}

impl RustyRaft {
    pub async fn new(node_id: String, server_address: String, clients_info: Vec<(String, String)>) -> Self {
        let clients = clients_info.iter().map(|info| Client::new(
            info.0.clone(),
            info.1.clone()
        )).collect();

        RustyRaft {
            node_id,
            color: RandomColor(pick_random_color()),
            server_address,
            clients,
            state: Arc::new(Mutex::new(State::new())),
            shutdown_tx: Arc::new(Mutex::new(None)),
            timeout_handler: Arc::new(TimeoutHandler::new()),
        }
    }

    pub fn log(&self, message: String) {
        let node_id = self.node_id.clone();
        let color = *self.color;
        println!("{}", format!("[Node {}]: {}", node_id, message).color(color).bold());
        // let state = Arc::clone(&self.state);
        // tokio::spawn(async move {
        //     let (role, term) = {
        //         let state = state.lock().await;
        //
        //         (state.role.clone(), state.current_term)
        //     };
        //
        //     println!("{}", format!("[Node {} ({:?} term {})]: {}", node_id, role, term, message).color(color).bold());
        // });
    }
}

// If we're handling all the cloning logic via Arc, we don't need to implement Clone
impl Clone for RustyRaft {
    fn clone(&self) -> Self {
        RustyRaft {
            node_id: self.node_id.clone(),
            color: RandomColor((*self.color).clone()),
            server_address: self.server_address.clone(),
            clients: self.clients.clone(),
            state: Arc::clone(&self.state),
            shutdown_tx: Arc::clone(&self.shutdown_tx),
            timeout_handler: Arc::clone(&self.timeout_handler),
        }
    }
}