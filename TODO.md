# TODO

I want to break this project down into steps and perform each step individually.

Here's ChatGPT's recommended project breakdown:

### Progress:
- [x] Step 1: Project Setup
- [x] Step 2: Tokio and Async Setup
- [ ] Step 3: Protobuf and gRPC Integration
- [ ] Step 4: Basic RAFT Protocol Impl
- ...

## Step 1: Project Setup and Environment Configuration
* Task: Set up the Rust environment and initialize a new Rust project using Cargo.
* Test: Ensure that the Rust toolchain and Cargo work by compiling a simple "Hello World" program.

## Step 2: Integrating Tokio and Basic Async Setup
* Task: Integrate the Tokio crate and set up a basic asynchronous environment.
* Test: Implement and test a simple asynchronous function to confirm Tokio is configured correctly.

## Step 3: Protobuf and gRPC Integration
* Task: Add dependencies for gRPC and Protobuf; set up basic RPC communication.
* Test: Create a dummy gRPC service and client to test basic communication.

## Step 4: Basic Raft Protocol Implementation
* Task: Implement the basic Raft protocol structures and states.
* Test: Develop unit tests for leader election and log replication at a very basic level.

## Step 5: Bazel Integration with Cargo Parity
* Task: Integrate Bazel using bzlmod, ensuring feature parity with Cargo.
* Test: Build and run tests using both Cargo and Bazel to ensure similar outcomes.

## Step 6: Full Raft Protocol Implementation with Async and gRPC
* Task: Complete the Raft consensus algorithm implementation using Tokio and gRPC.
* Test: Simulate a small cluster of Raft nodes and test for correct leader election, log replication, and failover.

## Step 7: Optimizations and Streaming
* Task: Implement streaming messages between nodes and optimize the system.
* Test: Benchmark the performance and ensure stability under high throughput and simulated network issues.

## Step 8: Advanced Bazel Features and Remote Execution
* Task: Configure Bazel for remote builds and execution with NativeLink.
* Test: Execute builds remotely and verify build times and correctness.

## Step 9: Documentation and Extra Credit Challenges
* Task: Document the project thoroughly and tackle extra credit challenges if time allows.
* Test: Review documentation for clarity and completeness. Test TLA+ specifications and performance enhancements.

## Step 10: Final Testing and Cleanup
* Task: Perform comprehensive testing across all nodes, configurations, and edge cases.
* Test: Conduct stress tests and fix any bugs or performance issues found.