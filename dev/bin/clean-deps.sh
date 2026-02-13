#!/usr/bin/env bash
set -euo pipefail

BASE=docker-compose.yml
OVERRIDE=docker-compose.docker.yml   # contains the extra_hosts entry
DAGSTER_FILE=docker-compose.dagster.yml
GOTENBERG_FILE=docker-compose.gotenberg.yml
JAEGER_FILE=docker-compose.jaeger.yml

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
FILES+=(-f "$DAGSTER_FILE")
if [[ "${GOTENBERG:-false}" == "true" ]]; then
    FILES+=(-f "$GOTENBERG_FILE")
fi
if [[ "${JAEGER:-false}" == "true" ]]; then
    FILES+=(-f "$JAEGER_FILE")
fi
[[ "$ENGINE" == docker ]] && FILES+=(-f "$OVERRIDE")   # extra_hosts only on Docker

# ── Down ────────────────────────────────────────────────────────────────────────
exec "$ENGINE" compose "${FILES[@]}" down -v -t 2
