.PHONY: build build-apple-silicon build-apple-intel build-linux-x86 build-linux-arm release clean

BINARY_NAME := ahab

# Default build for current platform
build:
	cargo build --release

# Apple Silicon (M1/M2/M3)
build-apple-silicon:
	cargo build --release --target aarch64-apple-darwin

# Apple Intel (x86_64)
build-apple-intel:
	cargo build --release --target x86_64-apple-darwin

# Linux x86_64 (requires cross or appropriate linker)
build-linux-x86:
	cross build --release --target x86_64-unknown-linux-gnu

# Linux ARM64 (requires cross or appropriate linker)
build-linux-arm:
	cross build --release --target aarch64-unknown-linux-gnu

# Build all release targets
release: build-apple-silicon build-apple-intel build-linux-x86 build-linux-arm
	mkdir -p dist
	cp target/aarch64-apple-darwin/release/$(BINARY_NAME) dist/$(BINARY_NAME)-aarch64-apple-darwin
	cp target/x86_64-apple-darwin/release/$(BINARY_NAME) dist/$(BINARY_NAME)-x86_64-apple-darwin
	cp target/x86_64-unknown-linux-gnu/release/$(BINARY_NAME) dist/$(BINARY_NAME)-x86_64-linux
	cp target/aarch64-unknown-linux-gnu/release/$(BINARY_NAME) dist/$(BINARY_NAME)-aarch64-linux

clean:
	cargo clean
	rm -rf dist