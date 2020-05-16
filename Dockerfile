ARG RUST_VERSION=1.43
FROM rust:${RUST_VERSION} AS build

ENV RUST_LOG=trace
EXPOSE 6060
WORKDIR /api
RUN rustup target add x86_64-unknown-linux-musl

RUN USER=root cargo new juniper-demo
WORKDIR /api/juniper-demo

COPY .env Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release
ENTRYPOINT [ "./target/release/juniper-demo" ]
