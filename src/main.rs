mod raft;

use std::env::args;
use std::io::ErrorKind::InvalidInput;
use raft::service::RustyRaft;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = args().collect();
    println!("Arguments: {:?}", args);
    if args.len() < 3 {
        println!("Args: <node_id> <server_address> <client_addresses>");

        return InvalidInput.into();
    }

    let node_id = args[1].clone();
    let addr = args[2].parse()?;
    let client_addresses = args[3].split(",").collect();
    let raft_service: RustyRaft = RustyRaft::new(
        node_id,
        addr,
        vec!["[::1]:50052".to_string()],
    ).await;

    raft_service.run_server().await?;

    Ok(())
}
