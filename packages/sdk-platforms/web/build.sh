#!/bin/bash

set -e

echo "🦀 Building zkSync SSO WASM SDK..."

# Navigate to the web SDK directory
cd "$(dirname "$0")"

# Ensure wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack is not installed. Please install it first:"
    echo "   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
fi

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf pkg-bundler pkg-node dist

# Build Rust crate for bundler target
echo "📦 Building WASM for bundler target..."
wasm-pack build ../rust/zksync-sso-erc4337/crates/zksync-sso-erc4337-ffi-web \
    --target bundler \
    --out-dir "../../../../web/pkg-bundler" \
    --scope zksync-sso

# Build Rust crate for Node.js target  
echo "🟢 Building WASM for Node.js target..."
wasm-pack build ../rust/zksync-sso-erc4337/crates/zksync-sso-erc4337-ffi-web \
    --target nodejs \
    --out-dir "../../../../web/pkg-node" \
    --scope zksync-sso

# Install npm dependencies
echo "📦 Installing npm dependencies..."
npm install

# Build TypeScript
echo "🔨 Building TypeScript..."
npm run build:ts

echo "✅ Build complete!"
echo ""
echo "📁 Generated files:"
echo "   - pkg-bundler/    (WASM for browsers)"
echo "   - pkg-node/       (WASM for Node.js)"
echo "   - dist/           (TypeScript compiled output)"
echo ""
echo "🚀 Ready to publish or use locally!"