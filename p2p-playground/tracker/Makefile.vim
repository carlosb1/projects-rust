.PHONY: test

build:
	cargo build

test: build
	RUST_LOG=info cargo test

run: build
	RUST_LOG=info cargo run
