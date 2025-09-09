FROM rust:1.86

WORKDIR /app
COPY . .

RUN cargo build --release --bin app

EXPOSE 3000

CMD ["./target/release/app"]
