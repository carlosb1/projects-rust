from rust:1.44 as build

RUN USER=root cargo new --bin bot
WORKDIR /bot
copy ./Cargo.lock ./Cargo.lock
copy ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs
COPY ./src ./src

RUN rm ./target/release/deps/bot_news*
RUN cargo build --release

FROM rust:1.44
COPY --from=build /bot/target/release/bot_news .
CMD ["./bot_news"]
