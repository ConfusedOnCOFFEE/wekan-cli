FROM rust:1.61

WORKDIR /usr/src/myapp
COPY crates .
WORKDIR /usr/src/myapp/wekan-cli
RUN cargo install --features wekan-cli/store --path .

# Use 'docker cp CONTAINER:/usr/local/cargo/bin/wekan-cli $PWD/wekan-cli
# to copy the binary to your local filesystem
