# Makefile for Rust + Metal GPU example

# Set the Cargo package name (must match Cargo.toml)
TARGET = multiply_gpu
BIN = target/release/$(TARGET)
LINK = ./metal_example   # symlink in project root

# Default: build, run, and create symlink
all: build link
	@echo "Running $(BIN) via symlink $(LINK)..."
	@$(LINK) 4

# Build release executable
build: $(BIN)

$(BIN):
	cargo build --release

# Create symlink to executable
link:
	@echo "Creating symlink $(LINK) -> $(BIN)"
	@ln -sf $(BIN) $(LINK)

# Clean build artifacts and symlink
clean:
	cargo clean
	@rm -f $(LINK)
