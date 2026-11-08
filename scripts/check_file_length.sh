#!/usr/bin/env sh
set -eu

limit=300
failed=0
for file in $(find crates -type f -name '*.rs' -not -path '*/target/*' | sort); do
    lines=$(awk 'END { print NR }' "$file")
    if [ "$lines" -gt "$limit" ]; then
        printf '%s: %s lines (limit %s)\n' "$file" "$lines" "$limit" >&2
        failed=1
    fi
done
exit "$failed"
