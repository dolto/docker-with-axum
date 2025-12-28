FROM rust:alpine3.23

RUN apk add --no-cache openssl-dev openssl-libs-static \
pkgconf musl-dev build-base \
&& cargo install sea-orm-cli \
&& rustup component add rustfmt
