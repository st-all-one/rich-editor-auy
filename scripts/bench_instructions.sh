#!/usr/bin/env sh
# Instruction gate: compares `Ir` counts against the saved baseline.
#
# The baseline is only written when it is absent, so a pull request without a baseline falls
# into the "save" branch once and every later run compares against `main`. The `soft_limits` of
# `benches/iai_hot.rs` make the runner exit non-zero (code 3) on a regression above 5% of `Ir`.
#
# Inside a container the runner needs `--security-opt seccomp=unconfined`: it calls `setarch` to
# disable ASLR, and Docker's default profile blocks `personality()`. The same flag is what
# `bench/ULTIMO.md` records for the verified local run.
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

if ! command -v valgrind >/dev/null 2>&1; then
    echo "valgrind ausente; bench de instruções pulado"
    exit 0
fi

cd "$root/bench"

if [ -f target/iai/main.json ]; then
    cargo bench --bench iai_hot -- --baseline=main
else
    cargo bench --bench iai_hot -- --save-baseline=main
fi
