#!/usr/bin/env bash
# sessionStart: inject mandatory project skills into agent context.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
OPBNB="$ROOT/.cursor/skills/reth-opbnb-port/SKILL.md"
RUST="$ROOT/.cursor/skills/rust-best-practices/SKILL.md"

missing=()
[[ -f "$OPBNB" ]] || missing+=("$OPBNB")
[[ -f "$RUST" ]] || missing+=("$RUST")
if ((${#missing[@]})); then
  msg="MANDATORY skills missing — stop and restore: ${missing[*]}"
  if command -v jq >/dev/null 2>&1; then
    jq -n --arg ctx "$msg" '{additional_context: $ctx}'
  else
    python3 -c 'import json,sys; print(json.dumps({"additional_context": sys.argv[1]}))' "$msg"
  fi
  exit 0
fi

PREAMBLE=$'MANDATORY SESSION START — skills loaded:\n1) reth-opbnb-port (PORT-PIPE + PORT-FLOW gate in plan.md)\n2) rust-best-practices (idiomatic Rust)\nName open PIPE and FLOW IDs before live/code. Follow both for all work in this repo.\n\n'

BODY="${PREAMBLE}"
BODY+=$'===== SKILL: reth-opbnb-port =====\n\n'
BODY+="$(cat "$OPBNB")"
BODY+=$'\n\n===== SKILL: rust-best-practices =====\n\n'
BODY+="$(cat "$RUST")"

if command -v jq >/dev/null 2>&1; then
  jq -n --arg ctx "$BODY" '{additional_context: $ctx}'
else
  ESCAPED=$(printf '%s' "$BODY" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))')
  printf '{"additional_context":%s}\n' "$ESCAPED"
fi
