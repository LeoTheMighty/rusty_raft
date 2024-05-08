use tokio::process::{Command, Child};
use std::process::Stdio;

async fn start_node(node_id: u32, address: String, client_addresses: Vec<String>) -> Child {
    Command::new("cargo")
        .args(&["run", "--quiet", "--", &node_id.to_string(), &address, &client_addresses.join(",")])
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
    let client_addresses = vec!["[::1]:50051|1".to_string(), "[::1]:50052|2".to_string(), "[::1]:50053|3".to_string(), "[::1]:50054|4".to_string(), "[::1]:50055|5".to_string()];
    // Start 5 nodes
    for id in 1..=5 {
        let address = format!("[::1]:{}", start_port + id);
        let client_addresses_clone: Vec<String> = client_addresses
            .clone()
            .into_iter()
            .filter(|s| s != &(address.clone() + id.to_string().as_str()))
            .collect();
        children.push(tokio::task::spawn(start_node(
            id,
            address,
            client_addresses_clone
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