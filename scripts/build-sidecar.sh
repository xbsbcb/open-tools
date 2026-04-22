#!/usr/bin/env bash
set -e
TRIPLE=$(rustc -vV | grep host | awk '{print $2}')
DEST="src-tauri/binaries/deno-${TRIPLE}"
mkdir -p src-tauri/binaries
deno compile --allow-read --allow-write \
  --output "$DEST" \
  plugin-host/main.ts
echo "Built: $DEST"
