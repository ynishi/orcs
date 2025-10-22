# Makefile for the orcs Rust workspace

# Use cargo as the command for all cargo-related tasks
CARGO := cargo

# Phony targets are not files. This prevents conflicts with files of the same name.
.PHONY: all build run check test fmt clippy clean

# The default target executed when you run `make`.
all: build

# Build the project in release mode with debug info.
build:
	$(CARGO) build

# Run the main application binary.
run:
	$(CARGO) run

# Check the project for errors without building executables.
check:
	$(CARGO) check

# Run all tests in the workspace.
test:
	$(CARGO) test

# Format all code in the workspace.
fmt:
	$(CARGO) fmt --all

# Lint the code using Clippy, treating all warnings as errors.
clippy:
	$(CARGO) clippy --all -- -D warnings

# Clean the project by removing the target directory.
clean:
	$(CARGO) clean

help:
	@echo "Available commands:"
	@echo "  make all      - Build the project (default)"
	@echo "  make build    - Build the project"
	@echo "  make run      - Run the application"
	@echo "  make check    - Check the code for errors"
	@echo "  make test     - Run all tests"
	@echo "  make fmt      - Format the code"
	@echo "  make clippy   - Lint the code"
	@echo "  make clean    - Clean build artifacts"
