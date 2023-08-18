.PHONY: test build

build:
	cargo build

test: build
	cargo test
