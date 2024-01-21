test:
	cargo test

web:
	cargo build --lib --release --target wasm32-unknown-unknown

serve: web
	python3 -m http.server

.PHONY: test serve web
