# ELFI Documentation

This directory contains the documentation for the ELFI (Event-sourcing Literate File Interpreter) project.

## Quick Start

### For New Developers

If you're setting up this project for the first time:

```bash
# Option 1: Using just (recommended)
just init

# Option 2: Manual setup
cargo install mdbook mdbook-mermaid
mdbook-mermaid install .
```

### Development Commands

```bash
# Serve documentation locally (auto-reload on changes)
just serve
# or
mdbook serve --open

# Build documentation
just build
# or 
mdbook build

# Generate merged markdown
just merge
# or
./merge_markdown.sh
```

## Project Structure

```
docs/
├── book.toml          # mdbook configuration
├── Cargo.toml         # Development tool dependencies
├── justfile           # Task runner (like npm scripts)
├── src/               # Markdown source files
│   ├── SUMMARY.md     # Book structure
│   ├── designs/       # Design documents
│   ├── implementations/ # Implementation docs
│   └── example.elf.md # Example files
├── merge_markdown.sh  # Script to merge all docs
└── README.md         # This file
```

## Features

- **Mermaid Diagrams**: Supports mermaid diagrams in markdown
- **Live Reload**: Auto-refresh during development  
- **Smart Merge**: Combines all docs into a single file
- **Code Highlighting**: Syntax highlighting for multiple languages
- **Cross-references**: Internal links and navigation

## Cargo vs uv Comparison

| Feature | Cargo | uv/Python |
|---------|-------|-----------|
| Dependency file | `Cargo.toml` | `pyproject.toml` |
| Install deps | `cargo install <tool>` | `uv sync` |
| Task runner | `just` or `cargo run` | `uv run` |
| Lock file | `Cargo.lock` | `uv.lock` |

**Key differences:**
- Cargo tools are typically installed **globally** (`cargo install`)  
- Python tools are often installed **per-project** (`uv add --dev`)
- For documentation tools, global installation is usually preferred

## Adding Dependencies

For documentation tools, update the `justfile` or use `cargo-make`:

```toml
# In Cargo.toml - just for documentation
[package.metadata.docs.tools]
mdbook = "0.4"
mdbook-mermaid = "0.15"
```

Then run:
```bash
just install-tools
```