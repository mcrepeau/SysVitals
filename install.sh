#!/bin/sh
set -e

# Configuration - Replace these with your own repo and binary names
REPO="mcrepeau/sysvitals"
BINARY="sysvitals"
INSTALL_DIR="/usr/local/bin"

echo "Installing $BINARY..."

# Detect OS
OS="$(uname | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

# Map architecture to GitHub asset naming (adjust as needed)
case "$ARCH" in
  x86_64) ARCH="x86_64" ;;
  amd64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

# Construct the asset name (example: sysvitals-v1.0.0-linux-x86_64.tar.gz)
get_latest_release() {
  curl --silent "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
}

TAG=$(get_latest_release)
if [ -z "$TAG" ]; then
  echo "Failed to get latest release tag"
  exit 1
fi

ASSET_NAME="${BINARY}-${TAG}-${OS}-${ARCH}.tar.gz"
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$TAG/$ASSET_NAME"

echo "Downloading $DOWNLOAD_URL..."
curl -L --fail --silent --show-error -o "/tmp/$ASSET_NAME" "$DOWNLOAD_URL"

echo "Extracting..."
tar -xzf "/tmp/$ASSET_NAME" -C /tmp

echo "Installing to $INSTALL_DIR..."
chmod +x "/tmp/$BINARY"
sudo mv "/tmp/$BINARY" "$INSTALL_DIR/"

echo "Cleaning up..."
rm "/tmp/$ASSET_NAME"

echo "$BINARY installed successfully to $INSTALL_DIR!"
echo "You can run it by typing '$BINARY' in your terminal."
