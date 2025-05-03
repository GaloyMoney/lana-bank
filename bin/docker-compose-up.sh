#!/usr/bin/env bash
set -euo pipefail

BASE=docker-compose.yml
OVERRIDE=docker-compose.docker.yml   # contains the extra_hosts entry

# ── Pick container engine ───────────────────────────────────────────────────────
if [[ -n "${ENGINE_DEFAULT:-}" ]]; then            # honour explicit choice
  ENGINE="$ENGINE_DEFAULT"
else                                               # otherwise prefer docker
  ENGINE=docker
fi

# ensure the binary is on PATH
if ! command -v "$ENGINE" >/dev/null 2>&1; then
  printf 'Error: requested engine "%s" not found in $PATH\n' "$ENGINE" >&2
  exit 1
fi

# ── Compose file set ────────────────────────────────────────────────────────────
FILES=(-f "$BASE")
[[ "$ENGINE" == docker ]] && FILES+=(-f "$OVERRIDE")   # extra_hosts only on Docker

# ── Up ──────────────────────────────────────────────────────────────────────────
exec "$ENGINE" compose "${FILES[@]}" up --wait -d "$@"
