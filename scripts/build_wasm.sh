#!/bin/sh

# Change to project root directory
cd "$(dirname "$0")/.."

wasm-pack build ../../wasm --out-dir ../server/axum/assets/wasm --target web
