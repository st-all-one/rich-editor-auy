#!/usr/bin/env sh
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$ROOT"

cargo build -p rich-editor-auy-cli
deno bundle --config packages/ts/deno.json --platform browser --output examples/demo/main.js examples/demo/main.ts
cp packages/ts/packages/core/wasm/rich_editor_auy_wasm_bg.wasm examples/demo/rich_editor_auy_wasm_bg.wasm

if command -v php >/dev/null 2>&1; then
  exec php -S 127.0.0.1:8787 -t examples/demo examples/demo/server.php
fi

exec docker run --rm -p 127.0.0.1:8787:8787 \
  -v "$ROOT":/app -w /app \
  php:8.4-cli php -S 0.0.0.0:8787 -t examples/demo examples/demo/server.php
