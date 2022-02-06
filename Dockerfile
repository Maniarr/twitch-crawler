FROM rust:1.58-slim-buster as builder

RUN apt-get update
RUN apt-get install -y libssl-dev build-essential zlib1g-dev 

COPY Cargo.toml /workspace/
COPY Cargo.lock /workspace/
COPY src /workspace/src
COPY twitch-rs/Cargo.toml /workspace/twitch-rs/
COPY twitch-rs/src /workspace/twitch-rs/src
COPY warp10.rs/Cargo.toml /workspace/warp10.rs/
COPY warp10.rs/src /workspace/warp10.rs/src

WORKDIR /workspace

RUN cargo build --release


FROM debian:buster-slim

RUN apt-get update && apt-get install -y ca-certificates

RUN update-ca-certificates --fresh

COPY --from=builder /workspace/target/release/twitch-crawler /usr/bin/twitch-crawler
COPY entrypoint.sh /

entrypoint ["bash", "/entrypoint.sh"]
