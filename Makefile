include: .env
export

watch:
	RUST_LOG=debug cargo watch -w src -x "lrun --bin ship"
seed:
	RUST_LOG=debug cargo run --bin seed
