include: .env
export

watch:
	RUST_LOG=debug cargo watch -w src -x "lrun --bin main"
seed:
	RUST_LOG=debug cargo lrun --bin seed