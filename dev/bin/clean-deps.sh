#!/usr/bin/env bash
set -euo pipefail

BASE=docker-compose.yml
DATA=docker-compose.data.yml
OVERRIDE=docker-compose.docker.yml   # contains the extra_hosts entry
DAGSTER_FILE=docker-compose.dagster.yml

# ── Pick container engine ───────────────────────────────────────────────────────
if [[ -n "${ENGINE_DEFAULT:-}" ]]; then
  ENGINE="$ENGINE_DEFAULT"
else
  ENGINE=docker
fi

if ! command -v "$ENGINE" >/dev/null 2>&1; then
  printf 'Error: requested engine "%s" not found in $PATH\n' "$ENGINE" >&2
  exit 1
fi

# ── Compose file set ────────────────────────────────────────────────────────────
FILES=(-f "$BASE")
FILES+=(-f "$DATA")
FILES+=(-f "$DAGSTER_FILE")
[[ "$ENGINE" == docker ]] && FILES+=(-f "$OVERRIDE")   # extra_hosts only on Docker

# ── Down ────────────────────────────────────────────────────────────────────────
exec "$ENGINE" compose "${FILES[@]}" down -v -t 2
