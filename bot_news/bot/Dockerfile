from rust:1.43
# RUN rustup update
RUN rustup default nightly
COPY . .
# RUN cargo clean
RUN cargo build
CMD cargo run --release
