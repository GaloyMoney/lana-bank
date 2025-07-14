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

# ── Pull images first (prevents concurrent map writes) ─────────────────────────
# Only pull in CI to avoid slow re-pulls during local development
if [[ "${CI:-false}" == "true" ]]; then
  echo "Pulling Docker images..."
  "$ENGINE" compose "${FILES[@]}" pull
fi

# ── Load environment variables ─────────────────────────────────────────────────
export TARGET_BIGQUERY_CREDENTIALS_JSON="$(echo $TF_VAR_sa_creds | base64 -d)"
echo $TARGET_BIGQUERY_CREDENTIALS_JSON > meltano/keyfile.json
export TARGET_BIGQUERY_DATASET="${TF_VAR_name_prefix}_dataset"

export DBT_BIGQUERY_DATASET="dbt_${TF_VAR_name_prefix}"
export DBT_BIGQUERY_PROJECT="$(echo $TF_VAR_sa_creds | base64 -d | jq -r '.project_id')"
export DOCS_BUCKET_NAME="${TF_VAR_name_prefix}-lana-documents"

export TARGET_BIGQUERY_LOCATION="US"
export DBT_BIGQUERY_KEYFILE="$(pwd)/meltano/keyfile.json"

# ── Up ──────────────────────────────────────────────────────────────────────────
echo "Starting services..."
"$ENGINE" compose "${FILES[@]}" up -d "$@"

wait4x postgresql ${PG_CON} --timeout 120s
