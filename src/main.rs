mod raft;

use std::env::args;
use raft::rusty_raft::RustyRaft;

use std::sync::Arc;

use tokio::io::{self, AsyncBufReadExt, BufReader};

use crate::raft::types::DynamicError;

#[tokio::main]
async fn main() -> Result<(), DynamicError> {
    let args: Vec<String> = args().collect();
    if args.len() < 3 {
        eprintln!("Usage: <node_id> <server_address> <clients>");
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Insufficient arguments").into());
    }

    let node_id = args[1].clone();
    let addr = args[2].parse()?;
    let clients_info: Vec<(String, String)> = if args.len() > 3 {
        args[3]
            .split(',')
            .map(|s| {
                let mut parts = s.split('|');
                (
                    parts.next().unwrap_or("").to_string(),
                    parts.next().unwrap_or("").to_string(),
                )
            })
            .collect()
    } else {
        Vec::new()
    };

    let raft_service = Arc::new(RustyRaft::new(
        node_id,
        addr,
        clients_info
    ).await);

    raft_service.clone().start_server().await;

    // Wait for other servers to start
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    raft_service.clone().reset_idle_timeout();

    // Spawn a task to listen for STDIN input and process commands
    let raft_service_clone = Arc::clone(&raft_service);
    tokio::spawn(async move {
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin).lines();

        while let Some(line) = reader.next_line().await.unwrap() {
            let trimmed_line = line.trim();
            match raft_service_clone.clone().handle_request(trimmed_line.to_string()).await {
                Ok(_) => {}
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    });

    tokio::signal::ctrl_c().await?;

    raft_service.stop_server().await;

    Ok(())
}
