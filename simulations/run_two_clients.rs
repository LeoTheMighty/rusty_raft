use tokio::process::{Command, Child};
use std::process::Stdio;

const NUM_CLIENTS: u32 = 2;

async fn start_node(node_id: u32, address: String, client_addresses: Vec<String>) -> Child {
    Command::new("cargo")
        .args(["run", "--quiet", "--", &node_id.to_string(), &address, &client_addresses.join(",")])
        .env("RUST_LOG", "info")
        .env("RUSTFLAGS", "-A warnings")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to start node process")
}

#[tokio::main]
async fn main() {
    let mut children = vec![];

    let start_port = 50050;
    // Start N nodes
    for id in 1..=NUM_CLIENTS {
        let address = format!("[::1]:{}", start_port + id);
        // Create the client addresses vector, skipping the current node's address
        let client_addresses: Vec<String> = (1..=NUM_CLIENTS)
            .filter(|&client_id| client_id != id)
            .map(|client_id| format!("[::1]:{}|{}", start_port + client_id, client_id))
            .collect();
        children.push(tokio::task::spawn(start_node(
            id,
            address,
            client_addresses
        )));
    }

    // Run for some time
    tokio::time::sleep(tokio::time::Duration::from_secs(40)).await;

    // Wait for other nodes to finish
    // Wait for other nodes to finish
    for handle in children {
        if let Ok(mut child) = handle.await {
            child.wait().await.expect("failed waiting for node");
        }
    }
}