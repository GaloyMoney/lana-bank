#!/bin/sh
set -eu

: "${DBT_BIGQUERY_PROJECT:?DBT_BIGQUERY_PROJECT not set — run from repo root with direnv}"
: "${TARGET_BIGQUERY_DATASET:?TARGET_BIGQUERY_DATASET not set — run from repo root with direnv}"
: "${STAGING_PROJECT:?STAGING_PROJECT not set — set to the staging GCP project ID (e.g. galoystaging)}"
: "${STAGING_DATASET:?STAGING_DATASET not set — set to the staging BigQuery dataset (e.g. galoy_staging_dataset)}"

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
