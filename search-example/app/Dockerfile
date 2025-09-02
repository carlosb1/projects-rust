FROM rust:1.86

WORKDIR /app
COPY . .

RUN cargo build --release --bin app


CMD ["./target/release/app"]
