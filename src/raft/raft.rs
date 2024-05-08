use std::sync::Arc;
use colored::Colorize;
use tokio::sync::{Mutex, watch};
use crate::raft::client::Client;
use crate::raft::color::{pick_random_color, RandomColor};
use crate::raft::time::TimeoutHandler;

#[derive(Default)]
pub struct RustyRaft {
    pub node_id: String,
    color: RandomColor,
    pub server_address: String,
    pub clients: Arc<Mutex<Vec<Client>>>,

    // Other fields
    pub shutdown_tx: Arc<Mutex<Option<watch::Sender<()>>>>,
    pub timeout_handler: TimeoutHandler,
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
            timeout_handler: TimeoutHandler::new(),
        }
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
            timeout_handler: self.timeout_handler.clone(),
        }
    }
}