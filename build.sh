#!/bin/bash

# 1) Make sure the wasm32 target is available
rustup target add wasm32-unknown-unknown

# 2) Compile your crate to Wasm (release for better optimizations)
cargo build --target wasm32-unknown-unknown --release

# 3) Run wasm-bindgen to generate JS glue in ./pkg
wasm-bindgen \
  target/wasm32-unknown-unknown/release/cardlayouter.wasm \
  --out-dir pkg \
  --target web \
  --no-typescript

echo "✅ Build complete. Serve index.html + pkg/ with any static server."
