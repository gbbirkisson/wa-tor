.PHONY: all clean check test fmt clippy lint build run
.DEFAULT_GOAL:=all

all: check test lint

clean:
	cargo clean

check:
	cargo check

test:
	cargo test

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy -- -D warnings

lint: fmt clippy

build:
	cargo build --release

run:
	cargo run --release
