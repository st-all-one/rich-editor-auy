#!/usr/bin/env sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
out="$root/packages/ts/packages/core/wasm"
mkdir -p "$out"
CARGO_PROFILE_RELEASE_STRIP=none cargo build -p rich-editor-auy-wasm --target wasm32-unknown-unknown --release --locked
wasm-bindgen "$root/target/wasm32-unknown-unknown/release/rich_editor_auy_wasm.wasm" \
    --target web --out-dir "$out" --out-name rich_editor_auy_wasm
if command -v wasm-opt >/dev/null 2>&1; then
    cp "$out/rich_editor_auy_wasm_bg.wasm" "$out/.wasm-unoptimized"
    if ! wasm-opt -Oz "$out/.wasm-unoptimized" -o "$out/rich_editor_auy_wasm_bg.wasm"; then
        mv "$out/.wasm-unoptimized" "$out/rich_editor_auy_wasm_bg.wasm"
        echo "wasm-opt falhou; artefato mantido sem otimização" >&2
        exit 1
    fi
    before=$(wc -c < "$out/.wasm-unoptimized")
    after=$(wc -c < "$out/rich_editor_auy_wasm_bg.wasm")
    if [ "$after" -gt "$before" ]; then
        echo "aviso: wasm-opt aumentou o artefato de $before para $after bytes" >&2
    fi
    rm -f "$out/.wasm-unoptimized"
elif [ "${AUY_REQUIRE_WASM_OPT:-0}" = "1" ]; then
    echo "wasm-opt ausente e AUY_REQUIRE_WASM_OPT=1" >&2
    exit 1
fi
if command -v b3sum >/dev/null 2>&1; then
    b3sum "$out/rich_editor_auy_wasm_bg.wasm" > "$out/rich_editor_auy_wasm_bg.wasm.blake3"
fi
