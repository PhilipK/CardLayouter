#!/bin/bash

set -euo pipefail

TARGET="wasm32-unknown-unknown"

if ! rustup target list --installed | grep -q "^${TARGET}$"; then
  rustup target add "${TARGET}"
fi

cargo build --target "${TARGET}" --release

rm -rf dist
mkdir -p dist/pkg

wasm-bindgen \
  target/${TARGET}/release/cardlayouter.wasm \
  --out-dir dist/pkg \
  --target web \
  --no-typescript

cp index.html dist/

echo "✅ Build complete."
