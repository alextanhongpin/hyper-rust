-include .env
export

RUST_LOG := info

format:
	@cargo +nightly fmt

start:
	@cargo run -- run

watch:
	@cargo watch -x 'run -- run'
