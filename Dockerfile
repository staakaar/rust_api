FROM lukemathwalker/cargo-chef:latest-rust-1.72.0 as chef

WORKDIR /rust_api

RUN apt update && apt install lld clang -y

FROM chef as planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder

COPY --from=planner /rust_api/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

ENV SQLX_OFFLINE true

RUN cargo build --release --bin rust_api

# Runtime stage
# FROM rust:1.72.0-slim AS runtime
# rust:1.72.0-alpine check out rust-musl-builder
FROM debian:bookworm-slim AS runtime

WORKDIR /rust_api

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /rust_api/target/release/rust_api rust_api

COPY configuration configuration

ENV APP_ENVIRONMENT production

ENTRYPOINT ["./rust_api"]

# docker build --tag rust_api --file Dockerfile .
# docker build rust_api
# docker run -p 8000:8000 rust_api