.PHONY: all lint fmt clippy test

all: lint test

lint:
	cargo fmt --check
	cargo clippy --workspace --all-targets

test:
	cargo test --workspace
