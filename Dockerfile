FROM rust:1.83 AS builder
WORKDIR /usr/src/cron-trigger
COPY . .
RUN apt update && apt install -y protobuf-compiler
RUN cargo install --path . --bin cron-trigger

FROM debian:bookworm-slim

RUN apt update && apt install -y openssl ca-certificates

COPY --from=builder /usr/local/cargo/bin/cron-trigger /usr/local/bin/cron-trigger
CMD ["cron-trigger"]
