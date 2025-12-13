# Makefile for the orcs Rust workspace

# Use cargo as the command for all cargo-related tasks
CARGO := cargo

# Phony targets are not files. This prevents conflicts with files of the same name.
.PHONY: all build run check test fmt clippy clean schema-generate dev tauri release-patch install-cli preflight

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

# Generate TypeScript type definitions from Rust schemas.
schema-generate:
	$(CARGO) run --bin orcs -- schema generate

# Run the Tauri desktop application in development mode.
dev:
	cd orcs-desktop && npm run tauri dev

# Build the Tauri desktop application.
tauri:
	cd orcs-desktop && npm run tauri build

# Bump the patch version without publishing, then create and push plain version tag.
release-patch:
	$(CARGO) release patch --execute --no-confirm --no-publish
	@VERSION=$$(grep -A 10 '\[workspace.package\]' Cargo.toml | grep '^version' | sed 's/version = "\(.*\)"/\1/'); \
	git tag -a "v$$VERSION" -m "Release v$$VERSION"; \
	git push origin main --tags

# Install orcs-cli binary to the system.
install-cli:
	$(CARGO) install --path crates/orcs-cli

# Run preflight checks before release.
preflight:
	@echo "🚀 Running preflight checks..."
	@echo ""
	@echo "📋 Step 1/7: Rust format check..."
	$(CARGO) fmt --all --check
	@echo "✅ Format check passed"
	@echo ""
	@echo "📋 Step 2/7: Rust clippy..."
	$(CARGO) clippy --all -- -D warnings
	@echo "✅ Clippy passed"
	@echo ""
	@echo "📋 Step 3/7: Rust check..."
	$(CARGO) check
	@echo "✅ Check passed"
	@echo ""
	@echo "📋 Step 4/7: Rust tests..."
	$(CARGO) test
	@echo "✅ Rust tests passed"
	@echo ""
	@echo "📋 Step 5/7: TypeScript type check..."
	cd orcs-desktop && npx tsc --noEmit
	@echo "✅ TypeScript type check passed"
	@echo ""
	@echo "📋 Step 6/7: TypeScript tests..."
	cd orcs-desktop && npm run test:run
	@echo "✅ TypeScript tests passed"
	@echo ""
	@echo "📋 Step 7/7: Schema generation check..."
	@if git diff --quiet crates/orcs-interaction/src/lib.rs orcs-desktop/src/bindings/generated.ts; then \
		echo "✅ Schema is up to date"; \
	else \
		echo "⚠️  Warning: Schema may be out of sync. Run 'make schema-generate' if needed."; \
	fi
	@echo ""
	@echo "✨ All preflight checks passed! Ready for release."

help:
	@echo "Available commands:"
	@echo "  make all             - Build the project (default)"
	@echo "  make build           - Build the project"
	@echo "  make run             - Run the application"
	@echo "  make check           - Check the code for errors"
	@echo "  make test            - Run all tests"
	@echo "  make fmt             - Format the code"
	@echo "  make clippy          - Lint the code"
	@echo "  make clean           - Clean build artifacts"
	@echo ""
	@echo "Development tools:"
	@echo "  make schema-generate - Generate TypeScript types from Rust schemas"
	@echo "  make dev             - Run Tauri app in development mode"
	@echo "  make tauri           - Build Tauri desktop application"
	@echo "  make preflight       - Run all checks before release (format, lint, test, types)"
	@echo "  make release-patch   - Bump patch version, commit, tag, and push (no publish)"
	@echo "  make install-cli     - Install orcs-cli binary to the system"
