use std::fs;
use std::path::Path;

fn main() {
    println!("Starting .proto compilation...");

    let proto_file = "./src/raft/proto/raft.proto";

    let src = Path::new("src/raft/proto/gen/raft.rs");
    let dst = Path::new("src/raft/protobufs.rs");

    tonic_build::configure()
        .build_server(true)
        .out_dir("./src/raft/proto/gen")
        .compile(&[proto_file], &["."])
        .unwrap_or_else(|e| panic!("protobuf compile error: {e}"));

    if src.exists() {
        fs::copy(src, dst).expect("Failed to copy protobufs file");
        println!("cargo:rerun-if-changed=src/proto/gen/raft_protobufs");
    }

    println!("cargo:rerun-if-changed={proto_file}");

    println!("Finished .proto compilation");
}
