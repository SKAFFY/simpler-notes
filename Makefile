UNAME_S := $(shell uname -s)

.PHONY: all build build-git build-all build-gui run-gui test test-git test-all \
        test-vault test-watcher test-verbose fmt fmt-check lint clean doc doc-open

all: build test

# Build
build:
	cargo build -p simpler-notes-core

build-git:
	cargo build -p simpler-notes-core --features git

build-all:
	cargo build --workspace

# GUI
build-gui:
ifeq ($(UNAME_S),Darwin)
	cargo build -p simpler-notes-gui --features metal
else
	cargo build -p simpler-notes-gui --features wayland
endif

run-gui: build-gui
ifeq ($(UNAME_S),Darwin)
	./target/debug/simpler-notes-gui
else
	XDG_SESSION_TYPE=Wayland WAYLAND_DISPLAY=wayland-0 ./target/debug/simpler-notes-gui
endif

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
