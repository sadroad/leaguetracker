FROM rust:1.70 AS chef
RUN cargo install cargo-chef --locked
WORKDIR app

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY ./src ./src
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY Cargo.toml Cargo.lock ./
COPY ./src ./src
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates libsqlite3-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/server ./
COPY players ./

ENTRYPOINT ["./server"]
