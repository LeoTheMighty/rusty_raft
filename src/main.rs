mod raft;

use std::env::args;
use raft::rusty_raft::RustyRaft;
use raft::client::Client;

use std::sync::Arc;

use tokio::io::{self};

use crate::raft::types::DynamicError;

#[tokio::main]
async fn main() -> Result<(), DynamicError> {
    let args: Vec<String> = args().collect();
    let command = args[1].clone();
    if command == "msg" {
        if args.len() < 3 {
            eprintln!("Usage: <run|msg> <node_id> <message>");
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Insufficient arguments").into());
        }
        let node_id = args[2].clone();

        let client = Client::new(
            format!("[::1]:5005{node_id}"),
            node_id.clone()
        );

        println!("Sending message (\"{}\") to node {}...", args[3].clone(), node_id);
        match client.send_client_message(args[3].clone()).await {
            Ok(response) => {
                println!("Response received: {:?}", response);
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }

        return Ok(());
    }

    if command != "run" {
        eprintln!("Usage: <run|msg> <args>");
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Unrecognized Command").into());
    }

    if args.len() < 4 {
        eprintln!("Usage: <run|msg> <node_id> <server_address> <clients>");
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Insufficient arguments").into());
    }

    let node_id = args[2].clone();
    let addr = args[3].parse()?;
    let clients_info: Vec<(String, String)> = if args.len() > 4 {
        args[4]
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
    // let raft_service_clone = Arc::clone(&raft_service);
    // tokio::spawn(async move {
    //     let stdin = io::stdin();
    //     let mut reader = BufReader::new(stdin).lines();
    //
    //     while let Some(line) = reader.next_line().await.unwrap() {
    //         let trimmed_line = line.trim();
    //         match raft_service_clone.clone().handle_request(trimmed_line.to_string()).await {
    //             Ok(_) => {}
    //             Err(e) => eprintln!("Error: {}", e),
    //         }
    //     }
    // });

    tokio::signal::ctrl_c().await?;

    raft_service.stop_server().await;

    Ok(())
}
