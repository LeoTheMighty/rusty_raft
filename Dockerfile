# TODO: TRIED TO GET BAZEL BUILD FULLY WORKING BUT COULDN'T
# FIGURE OUT WHY I KEEP GETTING THIS:
# /root/.cache/bazel/_bazel_root/73d643538f44db4b1220faadb2d2124d/external/rules_rust~0.40.0~rust~rust_host_tools/bin/cargo: 3: Syntax error: ")" unexpected

#FROM --platform=linux/amd64 rust:1.56 as builder
#WORKDIR /usr/src/myapp
#COPY . .
#
## Install Bazelisk and use it as Bazel
#RUN apt-get update && \
#    apt-get install -y wget && \
#    wget https://github.com/bazelbuild/bazelisk/releases/download/v1.15.0/bazelisk-linux-amd64 && \
#    chmod +x bazelisk-linux-amd64 && \
#    mv bazelisk-linux-amd64 /usr/bin/bazel
#
#RUN bazel build //:raft_client --verbose_failures --subcommands
#
#FROM debian:buster-slim as final
#COPY --from=builder /usr/src/myapp/bazel-bin/raft_client /usr/local/bin/raft_client
#
#ENTRYPOINT ["/usr/local/bin/raft_client"]

# Use a base image that matches the target environment of the built binary
FROM --platform=linux/arm64 debian:buster-slim as final

# Set the working directory
WORKDIR /app

# Copy the pre-built binary from your local build output directory into the Docker image
# Ensure the path to bazel-bin/raft_client is correct relative to the Docker context
COPY ./out/raft_client /app/raft_client

# Set the necessary permissions for the binary
RUN chmod +x /app/raft_client

# Specify the command to run your application
ENTRYPOINT ["/app/raft_client"]