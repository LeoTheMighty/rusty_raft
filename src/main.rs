mod raft;

use std::env::args;
use std::io;
use raft::raft::RustyRaft;

use std::sync::Arc;

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

    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    raft_service.clone().send_heartbeats_to_clients().await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

    raft_service.stop_server().await;

    Ok(())
}
