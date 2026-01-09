FROM rust:1.92.0

RUN cargo install cargo-watch && \
  apt update && apt install pkg-config libssl-dev -y \
  && cargo install dioxus-cli --version 0.7.2

WORKDIR /home/app
