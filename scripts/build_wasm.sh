#!/usr/bin/env sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
out="$root/packages/ts/packages/core/wasm"
mkdir -p "$out"
CARGO_PROFILE_RELEASE_STRIP=none cargo build -p rich-editor-auy-wasm --target wasm32-unknown-unknown --release --locked
wasm-bindgen "$root/target/wasm32-unknown-unknown/release/rich_editor_auy_wasm.wasm" \
    --target web --out-dir "$out" --out-name rich_editor_auy_wasm
if command -v wasm-opt >/dev/null 2>&1; then
    wasm-opt -Oz "$out/rich_editor_auy_wasm_bg.wasm" -o "$out/rich_editor_auy_wasm_bg.wasm"
fi
if command -v b3sum >/dev/null 2>&1; then
    b3sum "$out/rich_editor_auy_wasm_bg.wasm" > "$out/rich_editor_auy_wasm_bg.wasm.blake3"
fi
