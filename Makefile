.PHONY: all watch watchtest wtable

all: clean check test build doc

check:
	cargo fix
	cargo check
	cargo clippy
	cargo fmt
	cargo +nightly udeps
checkdeny:
	cargo deny check

doc:
	cargo doc --no-deps --document-private-items --open

clean: 
	cargo clean --doc
build: 
	cargo build
test:
	cargo test

publish: all
	# not published yet
	# cargo publish
