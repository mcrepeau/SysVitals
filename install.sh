#!/bin/sh
set -e

REPO="mcrepeau/SysVitals"
BINARY="sysvitals"
INSTALL_DIR="/usr/local/bin"

echo "Installing $BINARY..."

# Detect OS and architecture, map to Rust target triple
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
      aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
      *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    ;;
  Darwin)
    case "$ARCH" in
      x86_64)  TARGET="x86_64-apple-darwin" ;;
      arm64)   TARGET="aarch64-apple-darwin" ;;
      *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    ;;
  *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

# Get latest release tag from GitHub API
TAG=$(curl --silent "https://api.github.com/repos/$REPO/releases/latest" \
  | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$TAG" ]; then
  echo "Failed to get latest release tag"
  exit 1
fi

ASSET_NAME="${BINARY}-${TARGET}"
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$TAG/$ASSET_NAME"

echo "Downloading $DOWNLOAD_URL..."
curl -L --fail --silent --show-error -o "/tmp/$BINARY" "$DOWNLOAD_URL"

echo "Installing to $INSTALL_DIR..."
chmod +x "/tmp/$BINARY"
sudo mv "/tmp/$BINARY" "$INSTALL_DIR/$BINARY"

echo "$BINARY installed successfully to $INSTALL_DIR!"
echo "You can run it by typing '$BINARY' in your terminal."
