#!/usr/bin/env bash
set -euo pipefail

BASE=docker-compose.yml
OVERRIDE=docker-compose.docker.yml   # contains the extra_hosts entry
DAGSTER_FILE=docker-compose.dagster.yml

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
if [[ "${DAGSTER:-false}" == "true" ]]; then
    FILES+=(-f "$DAGSTER_FILE")
fi
[[ "$ENGINE" == docker ]] && FILES+=(-f "$OVERRIDE")   # extra_hosts only on Docker

# ── Pull images first (prevents concurrent map writes) ─────────────────────────
# Only pull in CI to avoid slow re-pulls during local development
if [[ "${CI:-false}" == "true" ]]; then
  echo "Pulling Docker images..."
  "$ENGINE" compose "${FILES[@]}" pull
fi

# ── Load environment variables ─────────────────────────────────────────────────
export TARGET_BIGQUERY_DATASET="${TARGET_BIGQUERY_DATASET:-${TF_VAR_name_prefix:-${USER}}_dataset}"
export DBT_BIGQUERY_DATASET="${DBT_BIGQUERY_DATASET:-dbt_${TF_VAR_name_prefix:-${USER}}}"
export DBT_BIGQUERY_PROJECT="${DBT_BIGQUERY_PROJECT:-$(echo "$TF_VAR_sa_creds" | base64 -d | jq -r '.project_id')}"
export DOCS_BUCKET_NAME="${DOCS_BUCKET_NAME:-${TF_VAR_name_prefix:-${USER}}-lana-documents}"
export TARGET_BIGQUERY_LOCATION="${TARGET_BIGQUERY_LOCATION:-US}"

# ── Up ──────────────────────────────────────────────────────────────────────────
echo "Starting services..."
"$ENGINE" compose "${FILES[@]}" up -d "$@" --build

wait4x postgresql ${PG_CON} --timeout 120s

# wait for keycloak to be ready
wait4x http http://localhost:8081/health/ready --timeout 120s

if [[ "${DAGSTER:-false}" == "true" ]]; then
  wait4x http http://localhost:3000/graphql --timeout 120s || true
fi
