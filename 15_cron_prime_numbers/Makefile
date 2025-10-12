
# all:
# 	for TARGET in \
# 	  aarch64-apple-darwin \
# 	  x86_64-apple-darwin \
# 	  x86_64-unknown-linux-musl \
# 	  aarch64-unknown-linux-musl \
# 	  x86_64-pc-windows-gnu; do
# 	
# 	  echo "=== Building for $TARGET ==="
# 	  cargo build --release --target $TARGET || echo "failed $TARGET"
# 	done
# 

# ----------------------------------------
# Multi-target Rust cross-build Makefile
# ----------------------------------------

# List of targets you want to build for
TARGETS := \
	aarch64-apple-darwin \
	x86_64-apple-darwin \
	x86_64-unknown-linux-musl \
	aarch64-unknown-linux-musl \
	x86_64-pc-windows-gnu

# Release binary name (edit this)
BINARY := prime_numbers

# Directory for packaged artifacts
DIST_DIR := dist

# ----------------------------------------
# Default target
# ----------------------------------------
all: $(TARGETS)
	@echo "\033[1;32m All targets built successfully!\033[0m"

# ----------------------------------------
# Pattern rule: build each target
# ----------------------------------------
$(TARGETS):
	@echo "\033[1;34m=== Building for $@ ===\033[0m"
	cargo build --release --target $@
	@mkdir -p $(DIST_DIR)
	@cp target/$@/release/$(BINARY) $(DIST_DIR)/$(BINARY)-$@$(suffix $@)
	@echo "\033[1;32m✔ Built $(BINARY)-$@\033[0m"

# ----------------------------------------
# Clean up all build artifacts
# ----------------------------------------
clean:
	cargo clean
	rm -rf $(DIST_DIR)
	@echo "\033[1;33m🧹 Cleaned all artifacts\033[0m"

# ----------------------------------------
# Optional helper: show configured targets
# ----------------------------------------
list:
	@echo "Available targets:"
	@for t in $(TARGETS); do echo "  - $$t"; done

#
# original handwritten
#

init:
# 	brew install rustup
# 	$(HOME)/.cargo/bin/rustup default stable
	brew install mingw-w64
	$(HOME)/.cargo/bin/rustup target add x86_64-pc-windows-gnu
	$(HOME)/.cargo/bin/rustup target add --toolchain stable x86_64-pc-windows-gnu

	brew install messense/macos-cross-toolchains/aarch64-unknown-linux-musl
	$(HOME)/.cargo/bin/rustup target add aarch64-unknown-linux-musl


# 	$(HOME)/.cargo/bin/rustup target add x86_64-unknown-linux-musl
# 	$(HOME)/.cargo/bin/rustup target add x86_64-unknown-linux-gnu
# 	$(HOME)/.cargo/bin/rustup target add aarch64-unknown-linux-gnu
# 	$(HOME)/.cargo/bin/rustup target add x86_64-apple-darwin

	brew install FiloSottile/musl-cross/musl-cross
	brew install messense/macos-cross-toolchains/aarch64-unknown-linux-gnu

build:
# cargo clean
	cargo build --release --target x86_64-pc-windows-gnu -v
	cargo build --release
