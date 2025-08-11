# Makefile for elfi project documentation

# Ensure cargo bin is in PATH
SHELL := /bin/bash
PATH  := $(HOME)/.cargo/bin:$(PATH)

.PHONY: help build-doc serve-doc clean-doc

help:
	@echo "Makefile for elfi project documentation"
	@echo ""
	@echo "Usage:"
	@echo "    make build-doc    - Build all documentation (mdBook and rustdoc)"
	@echo "    make serve-doc    - Serve the book and open it in a browser"
	@echo "    make clean-doc    - Clean up documentation artifacts"

# Build all documentation
build-doc: rustdoc mdbook

# Serve the book, opening it in a browser
serve-doc:
	@echo "Serving documentation at http://localhost:3000 ..."
	@mdbook serve docs --open

# Clean up documentation artifacts
clean-doc:
	@echo "Cleaning documentation artifacts..."
	@cargo clean
	@rm -rf docs/book

# --- Internal Targets ---

.PHONY: rustdoc mdbook

# Build the rustdoc API reference
rustdoc:
	@echo "Building Rust API documentation..."
	@cargo doc --no-deps

# Build the mdBook
mdbook:
	@echo "Building book..."
	@mdbook build docs