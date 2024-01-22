test:
	cargo test

clean:
	cargo clean

web:
	cargo build --lib --release --target wasm32-unknown-unknown

serve: web
	python3 -m http.server

WASM_PATH := target/wasm32-unknown-unknown/release/relations.wasm
WEB_DIST_PATH := ../LukeGrahamLandry.github.io/relalg
dist: web
	cp "$(WASM_PATH)" "$(WEB_DIST_PATH)/$(WASM_PATH)"
	cp "index.html" "$(WEB_DIST_PATH)/index.html"

target/bench.txt:
	cargo run --example gen_bench > target/bench.txt

.PHONY: test serve web dist clean target/bench.txt
