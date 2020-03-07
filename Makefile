-include .env
export

RUST_LOG := info

format:
	@cargo +nightly fmt

start:
	@cargo watch -x run
