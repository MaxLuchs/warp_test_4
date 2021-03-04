include: .env
export

watch:
	RUST_LOG=debug cargo watch -w src -x "run --bin ship"
seed:
	RUST_LOG=debug cargo run --bin seed
