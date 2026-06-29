#!/usr/bin/env bash
set -euo pipefail

REPO="suraniharsh/kairos"
BIN="kairos"

# Detect OS
OS="$(uname -s)"
case "$OS" in
  Linux)  os="unknown-linux-gnu" ;;
  Darwin) os="apple-darwin" ;;
  *)      echo "error: unsupported OS: $OS" >&2; exit 1 ;;
esac

# Detect architecture
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64)        arch="x86_64" ;;
  aarch64|arm64) arch="aarch64" ;;
  *)             echo "error: unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

TARGET="${arch}-${os}"

# Resolve latest version
VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
  | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$VERSION" ]; then
  echo "error: could not fetch latest release" >&2
  exit 1
fi

FILENAME="${BIN}-${VERSION}-${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${FILENAME}"

echo "Installing kairos ${VERSION} (${TARGET})..."

# Download to temp dir
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

curl -fsSL --progress-bar "$URL" -o "$TMP/$FILENAME"
tar -xzf "$TMP/$FILENAME" -C "$TMP"

BIN_PATH=$(find "$TMP" -name "$BIN" -type f | head -1)
if [ -z "$BIN_PATH" ]; then
  echo "error: binary not found in archive" >&2
  exit 1
fi

# Pick install directory
if [ -w /usr/local/bin ]; then
  INSTALL_DIR="/usr/local/bin"
else
  INSTALL_DIR="${HOME}/.local/bin"
  mkdir -p "$INSTALL_DIR"
fi

install -m 755 "$BIN_PATH" "${INSTALL_DIR}/${BIN}"

echo "Installed: ${INSTALL_DIR}/${BIN}"

if ! command -v "$BIN" &>/dev/null; then
  echo ""
  echo "Note: ${INSTALL_DIR} is not in your PATH."
  echo "Add this to your shell profile:"
  echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
fi
