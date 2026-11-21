#!/usr/bin/env bash
set -euo pipefail
REPO="AmazingCoderA/LionSSH"
BIN="lionssh"
VERSION="${1:-latest}"
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64) TARGET="x86_64-unknown-linux-gnu" ;;
  aarch64|arm64) TARGET="aarch64-unknown-linux-gnu" ;;
  *) echo "Unsupported arch: $ARCH"; exit 1 ;;
esac
if [ "$VERSION" = "latest" ]; then
  URL="https://github.com/$REPO/releases/latest/download/${BIN}-${TARGET}.tar.gz"
else
  URL="https://github.com/$REPO/releases/download/${VERSION}/${BIN}-${VERSION}-${TARGET}.tar.gz"
fi
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
echo "Downloading $URL"
curl -fL "$URL" -o "$TMP/pkg.tar.gz"
tar -xzf "$TMP/pkg.tar.gz" -C "$TMP"
DIR="$(find "$TMP" -maxdepth 1 -type d -name "${BIN}-*" | head -n1)"
install -Dm755 "$DIR/$BIN" "${HOME}/.local/bin/$BIN"
echo "Installed to ${HOME}/.local/bin/$BIN"

