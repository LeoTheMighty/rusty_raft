# RustyRaft
RustyRaft is an implementation of the Raft Consensus Algorithm, designed in Rust and utilizing modern tools such as tokio for asynchronous programming, protobuf for data serialization, and gRPC for inter-node communication. The goal of this project is to provide a robust, efficient, and scalable solution for managing a distributed cluster's state in a fault-tolerant manner.

## Features
- **Consensus Handling**: Ensures that the cluster maintains consensus despite node failures or network partitions.
- **Log Replication**: Efficiently replicates state changes across all cluster nodes to ensure consistency.
- **Leader Election**: Dynamically elects a new leader in case of the current leader's failure.
- **Fault Tolerance**: Handles node failures gracefully without losing the cluster state.
- **Scalability**: Supports scaling the cluster dynamically with minimal impact on ongoing operations.

## Getting Started
These instructions will get your copy of the project up and running on your local machine for development and testing purposes.

### Prerequisites
- Rust Programming Language: Installation guide
- Protocol Buffers Compiler (protoc): Installation guide

### Installing
1. Clone the repository

```bash
git clone https://github.com/yourusername/RustyRaft.git
cd RustyRaft
```

2. Build the project

```bash
cargo build --release
```

3. Run unit tests
```bash
cargo test
```

### Configuration
!!TODO!!
Modify the config.toml file in the root directory to adjust the configuration settings such as server ports, node addresses, and other operational parameters.

### Usage
To start a node:

```bash
cargo run -- run <node_id> <server_address> <clients_info>
```

To run a N-numbered node cluster simulation:
```bash
cargo run --bin run_n_clients -- <number_of_clients>
```

## API Documentation
The API for interacting with the Raft cluster is defined using protobuf. See proto/raft.proto for the gRPC service definitions.

## License
Distributed under the MIT License. See LICENSE for more information.

## Acknowledgments
- [Raft Website](https://raft.github.io/)
- [Raft Paper](https://raft.github.io/raft.pdf)
- Tokio Project
- Rust and the Rust community