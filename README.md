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

### Installing (Using Cargo)
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

### Installing (Using Bazel)

1. Clone the repo
2. Build the project

```bash
./bin/build
```

3. Then the binary should exist here

```bash
./out/raft_client
```

4. Alternatively, you can also build the 5 node simulation

```bash
./bin/build_sim
./out/run_n_clients
```

5. If you want to test out remote caching, create a `.bazelrc.local` file and add the contents of NativeLink's quickstart `.bazelrc` to configure. Then run the build again and watch the remote caching work

### Installing (Using Docker) TODO!
1. Clone the repo
2. Build the project using Bazel
3. Docker Build

```bash
./bin/docker_build
```

4. Docker Run
```bash
./bin/docker_run
```

(Can also do docker run -f or --force to force it to build as well!)
```bash
./bin/docker_run -f
./bin/docker_run --force
```

TODO!!! This does not work and returns this error:
```
exec /app/raft_client: exec format error
```

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

## Acknowledgments/References
- [Raft Website](https://raft.github.io/)
- [Raft Paper](https://raft.github.io/raft.pdf)
- [Blake's RustyRaft implementation](https://github.com/blakehatch/Rusty-Raft)
- Tokio Project
- Rust and the Rust community