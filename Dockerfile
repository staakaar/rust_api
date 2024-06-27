FROM rust:1.72.0

WORKDIR /rust_pro

RUN apt update && apt install lld xlang -y

COPY . .

ENV SQLX_OFFLINE true

RUN cargo build --release

ENTRYPOINT ["./target/release/rust_api"]

# docker build --tag rust_api --file Dockerfile .
# docker build rust_api
# docker run -p 8000:8000 rust_api