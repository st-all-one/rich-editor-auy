#!/usr/bin/env bash
# Exemplo mínimo REAL do knudge: funda um corpus temporário, grava, busca, cria tarefa,
# retoma contexto e verifica o contrato de máquina (--json).
#
# Não toca neste repositório: tudo acontece em `mktemp -d`, removido no EXIT.
# Requer `kd` no PATH (o binário do knudge). Saída observada: ver exemplos/README.md.
set -euo pipefail

if ! command -v kd >/dev/null 2>&1; then
    echo "kd ausente no PATH; instale antes de rodar este exemplo" >&2
    exit 1
fi

work="$(mktemp -d "${TMPDIR:-/tmp}/knudge-exemplo-XXXXXX")"
trap 'rm -rf "$work"' EXIT
cd "$work"
git init -q .
git config user.email exemplo@example.com
git config user.name Exemplo
# `kd sync` usa git commit; sem assinatura para o exemplo rodar em qualquer máquina.
git config commit.gpgsign false

echo "== projeto temporário: $work =="
echo "== kd init =="
kd init | sed -n '1,4p'

echo "== kd write (primeira nota) =="
kd write --summary "O editor precisa de um gate único de qualidade" \
    --type fact --tag processo --anchor crates/core/src/lib.rs

echo "== kd write (mesmo statement: idempotente) =="
kd write --summary "O editor precisa de um gate único de qualidade" --type fact

echo "== kd write (quase-duplicata: merge) =="
kd write --summary "O editor precisa de um gate único de qualidade e de performance" --type fact

echo "== kd ask =="
kd ask "gate de qualidade"

echo "== kd write --dry-run (não persiste) =="
kd write --summary "Seleção e IME precisam de testes de regressão" \
    --type decision --dry-run

echo "== kd task =="
epic="$(kd task new --summary "Fundação do editor" --scope epic --anchor plan/)"
echo "epic=$epic"
kd task new --summary "Escrever o gate de CI" --scope task --parent "$epic"
kd task list --universe

echo "== kd rewind (handoff) =="
kd rewind

echo "== kd doctor --fix =="
kd doctor --fix 2>&1 | tail -3

echo "== kd sync (commit de notas/ + eventos/) =="
kd sync

echo "== contrato de máquina: --json é JSON válido em stdout =="
kd ask "gate" --json 2>/dev/null | python3 -c \
    'import json,sys; d=json.load(sys.stdin); assert d["success"] is True; print("json ok:", d["command"], "hits=", len(d["data"]["hits"]))'

echo "== exit codes observados =="
set +e
kd ask --id nao_existe_000  >/dev/null 2>&1; echo "id ausente -> $?"
kd verbo_inexistente        >/dev/null 2>&1; echo "verbo inválido -> $?"
kd write --summary "" --type fact >/dev/null 2>&1; echo "statement vazio -> $?"
kd prime | head -1 >/dev/null; echo "EPIPE -> ${PIPESTATUS[0]}"
set -e

echo "== arquivos do corpus (verdade = notas/, resto derivado) =="
find .knudge/notas -name '*.md' | sort
echo "== ok =="
