#!/usr/bin/env sh
set -eu

root="packages/ts"
if [ ! -d "$root" ]; then
    exit 0
fi

if grep -RInE '(class=|style=|<style|insertRule|\.css)' "$root" \
    --include='*.ts' --include='*.tsx' --include='*.js' --include='*.json' \
    --exclude-dir='wasm' --exclude-dir='node_modules'; then
    printf 'zero-style gate failed: library-authored style hook found\n' >&2
    exit 1
fi
