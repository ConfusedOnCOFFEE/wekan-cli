FROM rust:1.61

WORKDIR /usr/src/myapp
COPY crates .
WORKDIR /usr/src/myapp/wekan-cli

FROM debian:buster-slim
RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/wekan-cli /usr/local/bin/wekan-cli
CMD ["wekan-cli"]
