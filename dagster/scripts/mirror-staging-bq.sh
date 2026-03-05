#!/bin/sh
set -eu

STAGING_PROJECT="galoystaging"
STAGING_DATASET="galoy_staging_dataset"

: "${DBT_BIGQUERY_PROJECT:?DBT_BIGQUERY_PROJECT not set — run from repo root with direnv}"
: "${TARGET_BIGQUERY_DATASET:?TARGET_BIGQUERY_DATASET not set — run from repo root with direnv}"

DEV_PROJECT="$DBT_BIGQUERY_PROJECT"
DEV_DATASET="$TARGET_BIGQUERY_DATASET"

echo "Mirroring ${STAGING_PROJECT}:${STAGING_DATASET} -> ${DEV_PROJECT}:${DEV_DATASET}"

tables=$(bq ls --format=json --project_id="$DEV_PROJECT" "${STAGING_PROJECT}:${STAGING_DATASET}" \
  | jq -r '.[] | select(.type == "TABLE") | .tableReference.tableId' \
  | grep -v '^_dlt_')

copied=0
for table in $tables; do
  echo "  Copying ${table}..."
  bq cp -f --project_id="$DEV_PROJECT" \
    "${STAGING_PROJECT}:${STAGING_DATASET}.${table}" \
    "${DEV_PROJECT}:${DEV_DATASET}.${table}"
  copied=$((copied + 1))
done

echo "Done. Copied ${copied} tables."
