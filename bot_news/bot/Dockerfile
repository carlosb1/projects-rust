from rust:1.43
# RUN rustup update
RUN rustup default nightly
COPY . .
# RUN cargo clean
RUN cargo build

# MEILISEARCH_ADDRESS="127.0.0.1:7700"
# RUN curl -i -X POST "$MEILISEARCH_ADDRESS/indexes" --data "{ \"name\": \"News\", \"uid\":\"news\" }"
CMD cargo run --release
