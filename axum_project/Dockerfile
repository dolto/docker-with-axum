FROM rust:1.92.0

RUN cargo install cargo-watch && \
  apt update && apt install pkg-config libssl-dev -y

WORKDIR /home/app

CMD ["cargo", "watch", "-x", "run"]
