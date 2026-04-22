#!/usr/bin/env bash
set -e
DENO_VERSION="1.44.4"
TRIPLE=$(rustc -vV 2>/dev/null | grep host | awk '{print $2}')
DEST="src-tauri/binaries/deno-${TRIPLE}"
mkdir -p src-tauri/binaries

OS=$(uname -s)
case "$OS" in
  Linux)  ZIP="deno-x86_64-unknown-linux-gnu.zip" ;;
  Darwin) ZIP="deno-x86_64-apple-darwin.zip" ;;
  *)      echo "Unsupported OS: $OS"; exit 1 ;;
esac

URL="https://github.com/denoland/deno/releases/download/v${DENO_VERSION}/${ZIP}"
echo "Downloading $URL ..."
curl -fL "$URL" -o /tmp/deno.zip
unzip -o /tmp/deno.zip deno -d /tmp/deno-extract
mv /tmp/deno-extract/deno "$DEST"
chmod +x "$DEST"
echo "Installed: $DEST"
