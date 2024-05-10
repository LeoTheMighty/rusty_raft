use tokio::process::{Command, Child};
use tokio::io::{self, AsyncBufReadExt};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use std::process::Stdio;
use std::path::PathBuf;
use std::sync::Arc;
use std::env;
use std::env::args;

async fn build_project() {
    // Build the project once
    let output = Command::new("cargo")
        .args(["build", "--release"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .await
        .expect("failed to build project");

    if !output.status.success() {
        panic!("cargo build failed");
    }
}

async fn start_node(binary_path: PathBuf, node_id: u32, address: String, client_addresses: Vec<String>) -> Child {
    Command::new(binary_path)
        .args([&node_id.to_string(), &address, &client_addresses.join(",")])
        .env("RUST_LOG", "info")
        .env("RUSTFLAGS", "-A warnings")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to start node process")
}

async fn restart_node(
    binary_path: PathBuf,
    node_id: u32,
    address: String,
    client_addresses: Vec<String>,
    children: Arc<Mutex<Vec<JoinHandle<Child>>>>
) {
    // Stop the node if it's running
    {
        let mut children_guard = children.lock().await;
        if let Some(handle) = children_guard.get_mut(node_id as usize - 1) {
            if let Ok(mut child) = handle.await {
                // Send the SIGINT signal
                let _ = child.kill().await;

                // Wait for the process to exit
                let _ = child.wait().await;
            }
        }
    }

    // Sleep for a moment before restarting
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Start the node again
    let new_handle = tokio::task::spawn(start_node(binary_path.clone(), node_id, address.clone(), client_addresses.clone()));

    // Update the child in the children vector
    {
        let mut children_guard = children.lock().await;
        children_guard[node_id as usize - 1] = new_handle;
    }

    println!("Node {} restarted.", node_id);
}

#[tokio::main]
async fn main() {
    let args = args().collect::<Vec<String>>();
    let num_clients = args[1].parse::<u32>().unwrap();

    let mut children = vec![];
    let children_arc = Arc::new(Mutex::new(vec![]));

    build_project().await;

    // Get the path to the built binary
    let mut binary_path = env::current_dir().expect("failed to get current directory");
    binary_path.push("target/release/raft_node");

    let start_port = 50050;
    // Start N nodes
    for id in 1..=num_clients {
        let address = format!("[::1]:{}", start_port + id);
        // Create the client addresses vector, skipping the current node's address
        let client_addresses: Vec<String> = (1..=num_clients)
            .filter(|&client_id| client_id != id)
            .map(|client_id| format!("[::1]:{}|{}", start_port + client_id, client_id))
            .collect();
        let child = tokio::task::spawn(start_node(
            binary_path.clone(),
            id,
            address.clone(),
            client_addresses.clone()
        ));
        children.push(child);
    }

    // Store the children in the Arc<Mutex<Vec<JoinHandle<Child>>>>
    {
        let mut children_guard = children_arc.lock().await;
        *children_guard = children;
    }


    // Listen for keystrokes
    let children_clone = Arc::clone(&children_arc);
    let binary_path_clone = binary_path.clone();
    tokio::spawn(async move {
        let stdin = io::stdin();
        let mut reader = io::BufReader::new(stdin).lines();

        while let Some(line) = reader.next_line().await.unwrap() {
            if line.trim() == "r" {
                println!("Which node to restart?");
                if let Some(node_id_str) = reader.next_line().await.unwrap() {
                    if let Ok(node_id) = node_id_str.trim().parse::<u32>() {
                        if node_id >= 1 && node_id <= num_clients {
                            let address = format!("[::1]:{}", start_port + node_id);
                            // Create the client addresses vector, skipping the current node's address
                            let client_addresses: Vec<String> = (1..=num_clients)
                                .filter(|&client_id| client_id != node_id)
                                .map(|client_id| format!("[::1]:{}|{}", start_port + client_id, client_id))
                                .collect();
                            restart_node(
                                binary_path_clone.clone(),
                                node_id,
                                address,
                                client_addresses,
                                Arc::clone(&children_clone)
                            ).await;
                        } else {
                            println!("Invalid node ID: {}", node_id);
                        }
                    } else {
                        println!("Invalid input. Please enter a valid node ID.");
                    }
                }
            }
        }
    });

    // Run for some time
    tokio::time::sleep(tokio::time::Duration::from_secs(40)).await;

    // Wait for other nodes to finish
    let mut children_guard = children_arc.lock().await;
    for handle in children_guard.drain(..) {
        if let Ok(mut child) = handle.await {
            child.wait().await.expect("failed waiting for node");
        }
    }
}