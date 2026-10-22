FROM rust:1.87 AS builder

# Refresh trust before Cargo downloads dependencies.
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/grpc-cron-trigger
COPY . .
RUN apt update && apt install -y protobuf-compiler
RUN cargo install --path . --bin grpc-cron-trigger --locked

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends openssl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/grpc-cron-trigger /usr/local/bin/grpc-cron-trigger
CMD ["grpc-cron-trigger"]
