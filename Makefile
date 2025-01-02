debug:
	cargo build
	cp target/debug/rudle ./rudle

release:
	cargo build --release
	cp target/release/rudle ./rudle

clean:
	cargo clean
