build:
	cargo install clap
	cargo add regex
	RUSTFLAGS="-D warnings" cargo build
	cargo build --release
# 	cargo clean
	cp target/debug/template template.darwin.dbg
	cp target/release/template template.darwin
	rm -rfv target
	find / | ./template.darwin
	