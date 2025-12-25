FROM rust:1.92.0

RUN cargo install cargo-watch

WORKDIR /home/app/axum-project

CMD ["cargo", "watch", "-x", "run"]
