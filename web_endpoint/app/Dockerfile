from rust:1.42
RUN rustup update
RUN rustup default nightly
COPY . .
ENV ROCKET_ENV production
RUN cargo clean
RUN cargo build
CMD cargo run
