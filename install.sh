#!/bin/sh
# Jankenoboe installer — downloads the pre-built binary for your platform.
# Usage: curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh

set -e

REPO="pandazy/jankenoboe"
INSTALL_DIR="${JANKENOBOE_INSTALL_DIR:-$HOME/.local/bin}"
BINARY_NAME="jankenoboe"

# Detect OS
OS="$(uname -s)"
case "$OS" in
  Linux)   OS_TAG="linux" ;;
  Darwin)  OS_TAG="macos" ;;
  *)       echo "Error: Unsupported OS: $OS"; exit 1 ;;
esac

# Detect architecture
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64)   ARCH_TAG="x86_64" ;;
  aarch64|arm64)   ARCH_TAG="aarch64" ;;
  *)               echo "Error: Unsupported architecture: $ARCH"; exit 1 ;;
esac

# Validate platform combination
if [ "$OS_TAG" = "macos" ] && [ "$ARCH_TAG" = "x86_64" ]; then
  echo "Error: macOS x86_64 (Intel) is not supported. Only Apple Silicon (aarch64) builds are available."
  exit 1
fi

ASSET_NAME="${BINARY_NAME}-${OS_TAG}-${ARCH_TAG}.tar.gz"

# Get latest release tag
echo "Fetching latest release..."
LATEST_TAG=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')

if [ -z "$LATEST_TAG" ]; then
  echo "Error: Could not determine latest release."
  exit 1
fi

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${ASSET_NAME}"

echo "Downloading ${BINARY_NAME} ${LATEST_TAG} for ${OS_TAG}-${ARCH_TAG}..."
echo "  ${DOWNLOAD_URL}"

# Create install directory
mkdir -p "$INSTALL_DIR"

# Download and extract
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

curl -fsSL "$DOWNLOAD_URL" -o "${TMPDIR}/${ASSET_NAME}"
tar -xzf "${TMPDIR}/${ASSET_NAME}" -C "$TMPDIR"
mv "${TMPDIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

echo ""
echo "✓ Installed ${BINARY_NAME} to ${INSTALL_DIR}/${BINARY_NAME}"
echo ""

# Check if install dir is in PATH
case ":$PATH:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    echo "⚠ ${INSTALL_DIR} is not in your PATH."
    echo "  Add this to your shell profile (~/.zshrc, ~/.bashrc, etc.):"
    echo ""
    echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
    echo ""
    ;;
esac

echo "Next steps:"
echo "  1. Set your database path:  export JANKENOBOE_DB=~/db/datasource.db"
echo "  2. Verify installation:     jankenoboe --help"