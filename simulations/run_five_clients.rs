use std::process::{Command, Child};
use std::thread;
use std::time::Duration;

fn start_node(node_id: u32, address: &String, client_addresses: Vec<String>) -> Child {
    Command::new("cargo")
        .args(["run", "--quiet", "--", &node_id.to_string(), address, &client_addresses.join(",")])
        .env("RUST_LOG", "info")
        .env("RUSTFLAGS", "-A warnings")
        .spawn()
        .expect("failed to start node process")
}

fn main() {
    let mut children = vec![];

    let start_port = 50050;
    let client_addresses = vec!["[::1]:50051".to_string(), "[::1]:50052".to_string(), "[::1]:50053".to_string(), "[::1]:50054".to_string(), "[::1]:50055".to_string()];
    // Start 5 nodes
    for id in 1..=5 {
        let address = format!("[::1]:{}", start_port + id);
        children.push(start_node(
            id,
            &address,
            client_addresses.clone().iter().filter(|s| !s.starts_with(&address)).map(|s| s.to_string()).collect(),
        ));
    }

    // Run for some time
    thread::sleep(Duration::from_secs(10));

    // Wait for other nodes to finish
    for mut child in children {
        child.wait().expect("failed waiting for node");
    }
}