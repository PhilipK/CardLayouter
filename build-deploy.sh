#!/bin/bash

set -e

# 1) Add target
rustup target add wasm32-unknown-unknown

# 2) Build
cargo build --target wasm32-unknown-unknown --release

# 3) Run wasm-bindgen
rm -rf dist
mkdir dist

wasm-bindgen \
  target/wasm32-unknown-unknown/release/cardlayouter.wasm \
  --out-dir dist/pkg \
  --target web \
  --no-typescript

# 4) Copy HTML
cp index.html dist/

echo "✅ Build complete."
