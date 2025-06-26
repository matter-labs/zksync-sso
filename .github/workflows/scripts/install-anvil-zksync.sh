#!/bin/bash

set -euo pipefail

REPO_URL="https://github.com/matter-labs/anvil-zksync.git"
RELEASE_VERSION="v0.6.3"
RELEASE_FILE_NAME="anvil-zksync-${RELEASE_VERSION}-aarch64-apple-darwin.tar.gz"
RELEASE_URL="https://github.com/matter-labs/anvil-zksync/releases/download/${RELEASE_VERSION}/${RELEASE_FILE_NAME}"
INSTALL_DIR="/usr/local/bin"
TEMP_DIR="$(mktemp -d)"

curl -L "$RELEASE_URL" -o "$TEMP_DIR/$RELEASE_FILE_NAME"

echo "Extracting anvil-zksync..."
tar -xzf "$TEMP_DIR/$RELEASE_FILE_NAME" -C "$INSTALL_DIR"

rm -rf "$TEMP_DIR"

echo "Verifying anvil-zksync installation..."
if command -v anvil-zksync >/dev/null 2>&1; then
  echo "anvil-zksync installed successfully!"
  anvil-zksync --version
else
  echo "Error: anvil-zksync not found in PATH" >&2
  exit 1
fi