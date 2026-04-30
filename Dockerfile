FROM rust:1.83 AS builder
WORKDIR /usr/src/grpc-cron-trigger
COPY . .
RUN apt update && apt install -y protobuf-compiler
RUN cargo install --path . --bin grpc-cron-trigger

FROM debian:bookworm-slim

RUN apt update && apt install -y openssl ca-certificates

COPY --from=builder /usr/local/cargo/bin/grpc-cron-trigger /usr/local/bin/grpc-cron-trigger
CMD ["grpc-cron-trigger"]
