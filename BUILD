load(
    "@rules_rust//rust:defs.bzl",
    "rust_binary",
    "rust_library",
)

# See MODULE.bazel's additive_build_file tag.
# Extra build file content from file

load("@rules_rust//crate_universe:defs.bzl", "crates_vendor")

crates_vendor(
    name = "vendor",
    cargo_lockfile = "//:Cargo.lock",
    generate_build_scripts = True,
    manifests = ["//:Cargo.toml"],
    mode = "remote",
    tags = ["manual"],
)

rust_binary(
    name = "raft_client",
    srcs = glob(["src/**/*.rs"]),
    crate_root = "src/main.rs",
    edition = "2021",
    deps = [
        # Generated with `bazel query "@crates//:*"`
       "@crates//:colored",
        "@crates//:prost",
        "@crates//:prost-types",
        "@crates//:protobuf",
        "@crates//:rand",
        "@crates//:serde",
        "@crates//:tokio",
        "@crates//:tonic"
    ],
)

rust_binary(
    name = "run_n_clients",
    srcs = glob(["simulations/**/*.rs"]),
    crate_root = "simulations/run_n_clients.rs",
    edition = "2021",
    deps = [
        "@crates//:colored",
        "@crates//:prost",
        "@crates//:prost-types",
        "@crates//:protobuf",
        "@crates//:rand",
        "@crates//:serde",
        "@crates//:tokio",
        "@crates//:tonic"
    ],
)