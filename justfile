# Root justfile to orchestrate all project tasks

# Build the main Rust application
build:
    cargo build

# Build the main Rust application in release mode
build-release:
    cargo build --release

# Run tests for the main application
test:
    cargo test

# Build the documentation
# This command changes into the 'docs' directory and runs the 'build' command from its own justfile
docs-build:
    (cd docs && just build)

# Serve the documentation locally
# This command changes into the 'docs' directory and runs the 'serve' command from its own justfile
docs-serve:
    (cd docs && just serve)

# Clean all build artifacts for both the main project and the documentation
clean:
    @echo "Cleaning main project artifacts..."
    cargo clean
    @echo "Cleaning documentation artifacts..."
    (cd docs && just clean)
    @echo "✓ All clean."

