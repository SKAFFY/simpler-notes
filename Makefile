UNAME_S := $(shell uname -s)

.PHONY: all build build-mcp build-git build-all build-gui run-gui \
        test test-mcp test-e2e test-git test-all test-vault test-watcher test-verbose \
        coverage fmt fmt-check lint clean doc doc-open

all: build-mcp test-all

# Build
build:
	cargo build -p simpler-notes-core

build-mcp:
	cargo build -p simpler-notes-mcp

build-all:
	cargo build -p simpler-notes-core -p simpler-notes-mcp

# GUI
build-gui:
ifeq ($(UNAME_S),Linux)
	cargo build -p simpler-notes-gui --features linux
else ifeq ($(UNAME_S),Darwin)
	cargo build -p simpler-notes-gui --features macos
else
	cargo build -p simpler-notes-gui --features windows
endif

run-gui: build-gui
ifeq ($(UNAME_S),Darwin)
	./target/debug/simpler-notes-gui
else ifeq ($(UNAME_S),Linux)
	XDG_SESSION_TYPE=Wayland WAYLAND_DISPLAY=wayland-0 ./target/debug/simpler-notes-gui
else
	./target/debug/simpler-notes-gui
endif

# Test
test:
	cargo test -p simpler-notes-core

test-mcp:
	cargo test -p simpler-notes-mcp

test-e2e:
	cargo test -p simpler-notes-mcp --test e2e

test-git:
	cargo test -p simpler-notes-core --features git

test-all:
	cargo test -p simpler-notes-core -p simpler-notes-mcp

test-vault:
	cargo test -p simpler-notes-core vault

test-watcher:
	cargo test -p simpler-notes-core watcher

# Run specific tests with output
test-verbose:
	cargo test -p simpler-notes-core -- --nocapture

# Coverage
coverage:
	cargo tarpaulin -p simpler-notes-core -p simpler-notes-mcp --out Stdout

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
