debug:
	cargo build
	cp target/debug/rurdle ./rurdle

release:
	cargo build --release
	cp target/release/rurdle ./rurdle

clean:
	cargo clean
