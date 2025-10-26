default:
    @just --list

# Install navi locally
install:
    cargo install --path .

# Uninstall navi
uninstall:
    cargo uninstall navi

# Build the project
build:
    cargo build

# Build release version
build-release:
    cargo build --release

# Run all tests
test: test-sh test-cargo

# Run shell tests only
test-sh: build
    ./tests/run.sh

# Run cargo tests only
test-cargo:
    cargo test

# Run lints (fmt + clippy)
lint:
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features -- -D warnings

# Fix formatting and linting issues
fix:
    cargo fmt --all
    cargo clippy --all-targets --all-features --fix --allow-dirty

# Run cargo fmt
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Run clippy
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run clippy with fixes
clippy-fix:
    cargo clippy --all-targets --all-features --fix --allow-dirty

# Clean build artifacts
clean:
    cargo clean

# Update dependencies
update:
    cargo update

# Check for outdated dependencies
outdated:
    cargo outdated

# Run cargo check
check:
    cargo check

# Build documentation
doc:
    cargo doc --no-deps --open

# Run a development build and install
dev-install: build
    cargo install --path . --debug

# Show project info
info:
    @echo "Project: navi"
    @echo "Rust version: $(rustc --version)"
    @echo "Cargo version: $(cargo --version)"

# Run security audit
audit:
    cargo audit

# Generate code coverage (requires cargo-tarpaulin)
coverage:
    cargo tarpaulin --out Html --output-dir coverage

# Watch for changes and run tests (requires cargo-watch)
watch:
    cargo watch -x test

# Benchmark (requires nightly)
bench:
    cargo bench

# Release build for specific target
release target:
    ./scripts/release {{ target }}
