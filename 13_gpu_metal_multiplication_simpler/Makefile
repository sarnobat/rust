# Makefile for Rust + Metal GPU example

# Cargo target name (must match Cargo.toml)
TARGET = multiply_gpu

# Output paths
ARM64_BIN = target/arm64/release/$(TARGET)
X86_BIN   = target/x86_64/release/$(TARGET)
UNIVERSAL_BIN = target/universal/$(TARGET)

# Symlink name in project root
LINK = ./metal_example

# Default: build universal binary, create symlink, run
all: universal link
	@echo "Running $(UNIVERSAL_BIN) via symlink $(LINK)..."
	@$(LINK) 4

# Build ARM64 slice
$(ARM64_BIN):
	cargo build --release --target aarch64-apple-darwin
	mkdir -p target/arm64/release
	cp target/aarch64-apple-darwin/release/$(TARGET) $(ARM64_BIN)

# Build x86_64 slice
$(X86_BIN):
	cargo build --release --target x86_64-apple-darwin
	mkdir -p target/x86_64/release
	cp target/x86_64-apple-darwin/release/$(TARGET) $(X86_BIN)

# Combine into universal binary
universal: $(ARM64_BIN) $(X86_BIN)
	@echo "Creating universal binary..."
	mkdir -p target/universal
	lipo -create -output $(UNIVERSAL_BIN) $(ARM64_BIN) $(X86_BIN)
	@echo "Created: $(UNIVERSAL_BIN)"

# Symlink pointing to universal binary
link:
	@echo "Creating symlink $(LINK) -> $(UNIVERSAL_BIN)"
	@ln -sf $(UNIVERSAL_BIN) $(LINK)

# Clean build artifacts
clean:
	cargo clean
	rm -f $(LINK) $(ARM64_BIN) $(X86_BIN) $(UNIVERSAL_BIN)
