# Check for cargo installation
check-cargo:
    #!/usr/bin/env sh
    if ! command -v cargo >/dev/null 2>&1; then
        echo "cargo is not installed. Please install Rust:"
        echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi

# Build the Rust binary in release mode
build: check-cargo
    cargo build --release

# Install binary to bin/ directory for npm
install: build
    mkdir -p bin
    cp target/release/pretty-node bin/pretty-node

# Run tests
test: check-cargo
    cargo test

# Clean build artifacts
clean: check-cargo
    cargo clean
    rm -rf bin/pretty-node

# Run the CLI locally (after build)
run ARGS='--help': build
    ./target/release/pretty-node {{ARGS}}

# Test with a real package
test-express: build
    ./target/release/pretty-node tree express --quiet

# Test signature extraction
test-sig: build
    ./target/release/pretty-node sig express:Router --quiet

# Prepare for npm publishing
prep-publish: install
    @echo "Ready for: npm publish"

# Format code
fmt: check-cargo
    cargo fmt

# Run clippy lints
lint: check-cargo
    cargo clippy -- -D warnings

# Run all checks
check: fmt lint test
    @echo "All checks passed!"