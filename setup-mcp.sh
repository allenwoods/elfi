#!/bin/bash

# ELFI MCP Setup Script for Context7
set -e

echo "Setting up MCP for ELFI project with Context7..."

# Check for required tools
check_command() {
    if ! command -v "$1" &> /dev/null; then
        echo "Error: $1 is not installed. Please install it first."
        exit 1
    fi
}

# Check prerequisites
check_command node
check_command npm
check_command cargo

# Create necessary directories
mkdir -p elfi-storage
mkdir -p certs
mkdir -p .mcp

# Install MCP servers
echo "Installing MCP servers..."
npm install -g @context7/mcp-server
npm install -g @modelcontextprotocol/server-filesystem
npm install -g @modelcontextprotocol/server-github
npm install -g @modelcontextprotocol/server-memory
npm install -g @modelcontextprotocol/server-postgres

# Set up environment file
if [ ! -f .env ]; then
    cp .env.example .env
    echo "Created .env file. Please update it with your API keys."
fi

# Install Rust dependencies for ELFI
echo "Installing Rust dependencies..."
if [ ! -f Cargo.toml ]; then
    cat > Cargo.toml << 'EOF'
[workspace]
members = [
    "elfi-core",
    "elfi-parser",
    "elfi-cli",
    "elfi-ffi"
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["ELFI Team"]
license = "MIT OR Apache-2.0"

[workspace.dependencies]
automerge = "0.5"
zenoh = "0.11"
tree-sitter = "0.20"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
EOF
fi

# Create basic project structure
mkdir -p elfi-core/src
mkdir -p elfi-parser/src
mkdir -p elfi-cli/src
mkdir -p elfi-ffi/src

# Initialize elfi-core
if [ ! -f elfi-core/Cargo.toml ]; then
    cat > elfi-core/Cargo.toml << 'EOF'
[package]
name = "elfi-core"
version.workspace = true
edition.workspace = true

[dependencies]
automerge.workspace = true
zenoh.workspace = true
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
EOF
fi

# Initialize elfi-parser
if [ ! -f elfi-parser/Cargo.toml ]; then
    cat > elfi-parser/Cargo.toml << 'EOF'
[package]
name = "elfi-parser"
version.workspace = true
edition.workspace = true

[dependencies]
tree-sitter.workspace = true
serde.workspace = true
EOF
fi

# Initialize elfi-cli
if [ ! -f elfi-cli/Cargo.toml ]; then
    cat > elfi-cli/Cargo.toml << 'EOF'
[package]
name = "elfi-cli"
version.workspace = true
edition.workspace = true

[dependencies]
elfi-core = { path = "../elfi-core" }
elfi-parser = { path = "../elfi-parser" }
clap = { version = "4", features = ["derive"] }
tokio.workspace = true
EOF
fi

# Initialize elfi-ffi
if [ ! -f elfi-ffi/Cargo.toml ]; then
    cat > elfi-ffi/Cargo.toml << 'EOF'
[package]
name = "elfi-ffi"
version.workspace = true
edition.workspace = true

[lib]
crate-type = ["cdylib", "staticlib"]

[dependencies]
elfi-core = { path = "../elfi-core" }
EOF
fi

# Verify MCP configuration
echo "Verifying MCP configuration..."
if [ -f mcp.json ]; then
    echo "✓ MCP configuration found"
else
    echo "✗ MCP configuration missing"
    exit 1
fi

# Test Context7 connection (if API key is set)
if [ -f .env ]; then
    source .env
    if [ ! -z "$CONTEXT7_API_KEY" ] && [ "$CONTEXT7_API_KEY" != "your_context7_api_key_here" ]; then
        echo "Testing Context7 connection..."
        npx @context7/mcp-server test 2>/dev/null && echo "✓ Context7 connection successful" || echo "✗ Context7 connection failed"
    else
        echo "⚠ Context7 API key not configured. Please update .env file."
    fi
fi

echo ""
echo "MCP setup complete! Next steps:"
echo "1. Update .env with your API keys"
echo "2. Run 'cargo build' to compile the ELFI kernel"
echo "3. Start the MCP servers with Claude Code"
echo ""
echo "To use with Claude Code:"
echo "- Ensure mcp.json is in your project root"
echo "- Restart Claude Code to load the MCP configuration"