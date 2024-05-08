mod raft;

use std::env::args;
use std::io;
use raft::service::RustyRaft;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = args().collect();
    println!("Arguments: {:?}", args);
    if args.len() < 3 {
        eprintln!("Usage: <node_id> <server_address> <client_addresses>");
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Insufficient arguments").into());
    }

    let node_id = args[1].clone();
    let addr = args[2].parse()?;
    let client_addresses: Vec<String> = args[3].split(",").map(|s| s.to_string()).collect();
    let raft_service: RustyRaft = RustyRaft::new(
        node_id,
        addr,
        client_addresses
    ).await;

    raft_service.start_server().await;

    raft_service.send_message_to_clients("Hello World!".to_string()).await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    raft_service.stop_server().await;

    Ok(())
}
