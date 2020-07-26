from rust:1.44 as build
RUN rustup toolchain add nightly
RUN rustup default nightly
RUN USER=root cargo new --bin web_app
WORKDIR /web_app
copy ./Cargo.lock ./Cargo.lock
copy ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs
COPY ./src ./src

RUN rm ./target/release/deps/main*
RUN cargo build --release

FROM rust:1.44
COPY --from=build /web_app/target/release/main .
CMD ["./main"]
