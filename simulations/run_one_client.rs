mod run_five_clients;

use std::process::{Command, Child};
use std::thread;
use std::time::Duration;

fn start_node(node_id: u32) -> Child {
    Command::new("cargo")
        .args(["run", "--", &node_id.to_string(), "[::1]:50051"])
        .spawn()
        .expect("failed to start node process")
}

fn main() {
    let mut child = start_node(1);

    // Run for some time
    thread::sleep(Duration::from_secs(10));

    // Wait for other nodes to finish
    child.wait().expect("failed waiting for node");
}