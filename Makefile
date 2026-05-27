.PHONY: all build build-git test test-git test-vault clean

all: build test

# Build
build:
	cargo build -p simpler-notes-core

build-git:
	cargo build -p simpler-notes-core --features git

build-all:
	cargo build --workspace

# Test
test:
	cargo test -p simpler-notes-core

test-git:
	cargo test -p simpler-notes-core --features git

test-all:
	cargo test --workspace

test-vault:
	cargo test -p simpler-notes-core vault

test-watcher:
	cargo test -p simpler-notes-core watcher

# Run specific tests with output
test-verbose:
	cargo test -p simpler-notes-core -- --nocapture

# Format
fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

# Lint
lint:
	cargo clippy --all-targets -- -D warnings

# Clean
clean:
	cargo clean

# Docs
doc:
	cargo doc --no-deps -p simpler-notes-core

doc-open:
	cargo doc --no-deps -p simpler-notes-core --open
