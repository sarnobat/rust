build:
	cargo install clap
	cargo add regex
	RUSTFLAGS="-D warnings" cargo build
	cargo build --release
	cargo fmt
# 	cargo clean
	cp target/debug/template template.darwin.dbg
	cp target/release/template template.darwin
	rm -rfv target
	find / | RUST_BACKTRACE=1 ./template.darwin
	