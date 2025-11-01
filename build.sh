#!/bin/bash

set -euo pipefail

TARGET="wasm32-unknown-unknown"

if ! rustup target list --installed | grep -q "^${TARGET}$"; then
  rustup target add "${TARGET}"
fi

cargo build --target "${TARGET}" --release

wasm-bindgen \
  target/${TARGET}/release/cardlayouter.wasm \
  --out-dir pkg \
  --target web \
  --no-typescript

echo "✅ Build complete. Serve index.html + pkg/ with any static server."
