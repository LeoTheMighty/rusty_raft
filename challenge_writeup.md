# Rusty Raft Challenge
### Written graciously by Blake Hatch

The task is to implement the Raft Consensus Algorithm in Rust, made blazingly fast through efficient usage of Tokio async and gRPC together.

## Requirements
- Your implementation must make efficient use of tokio async.
- You must integrate bazel within the project and maintain feature parity with cargo for builds and tests.
    - You must use bzlmod as opposed to the standard WORKSPACE setup.
- The raft nodes must communicate through gRPC.
## Required Crates
- tokio
- serde
- prost
- prost-types
- tonic
- protobuf
- prost-build and/or tonic-build

## Extra credit
- [ ] Make your rust implementation match the TLA+ Spec.
- [ ] Make your Bazel setup remotely execute with NativeLink.
- [ ] Stream messages between nodes using protobuf and tokio together for improved performance. Ex:
```protobuf
service RaftServiceStream {
  rpc SendRaftMessage(RaftMessage) returns (stream RaftMessage);
}
```

## Reference Implementations for RAFT
- https://github.com/simple-raft-rs/raft-rs
- https://github.com/async-raft/async-raft
## Reference Integrations for Bazel Rust setups with Bzlmod *
- https://github.com/TraceMachina/nativelink
- https://github.com/blakehatch/Rusty-Raft
  *Note: Do not look at the Bazel setups until you have laid out your project structure with just cargo. 
