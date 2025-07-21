build:
	cargo install clap
	cargo add regex
	RUSTFLAGS="-D warnings" cargo build
# 	cargo clean
	cp /Volumes/git/github/depot_parent/rust/05_template/target/debug/template template.darwin
	rm -rfv target
	find / | ./template.darwin
	