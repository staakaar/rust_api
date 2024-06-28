# Builder stage
FROM rust:1.72.0 AS builder

WORKDIR /rust_api

RUN apt update && apt install lld clang -y

COPY . .

ENV SQLX_OFFLINE true

RUN cargo build --release

# Runtime stage
FROM rust:1.72.0-slim AS runtime

WORKDIR /rust_api

COPY --from=builder /app/target/release/rust_api rust_api

COPY configuration configuration

ENV APP_ENVIRONMENT production

ENTRYPOINT ["./rust_api"]

# docker build --tag rust_api --file Dockerfile .
# docker build rust_api
# docker run -p 8000:8000 rust_api