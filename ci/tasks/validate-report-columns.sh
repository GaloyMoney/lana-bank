#!/bin/bash
set -euo pipefail

# Validate that expected_columns in reports.yml match dbt manifest columns.
#
# Generates the dbt manifest with dummy credentials (no BQ connection needed),
# then checks each report with expected_columns against the manifest.
#
# For supports_as_of reports, as_of_date is excluded from manifest columns
# since it is filtered out at query time.

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DOCKERFILE="$REPO_ROOT/dagster/Dockerfile"
DBT_PROJECT_DIR="$REPO_ROOT/dagster/src/dbt_lana_dw"
REPORTS_YML="$REPO_ROOT/dagster/generate_es_reports/reports.yml"

# Extract dbt package versions from Dockerfile to stay in sync
DBT_CORE_VERSION=$(grep -oP 'dbt-core~=\K[0-9.]+' "$DOCKERFILE" | head -1)
DBT_BQ_VERSION=$(grep -oP 'dbt-bigquery~=\K[0-9.]+' "$DOCKERFILE" | head -1)

echo "Installing dbt-core~=${DBT_CORE_VERSION} dbt-bigquery~=${DBT_BQ_VERSION}..."
pip install -q "dbt-core~=${DBT_CORE_VERSION}" "dbt-bigquery~=${DBT_BQ_VERSION}" pyyaml

# Generate dbt manifest
echo "Installing dbt dependencies..."
dbt deps --project-dir "$DBT_PROJECT_DIR" --profiles-dir "$DBT_PROJECT_DIR"

echo "Generating dbt manifest..."
DBT_BIGQUERY_PROJECT=build-placeholder \
DBT_BIGQUERY_DATASET=build-placeholder \
DBT_BIGQUERY_CREDENTIALS_JSON='{"type":"service_account","project_id":"x"}' \
  dbt parse --project-dir "$DBT_PROJECT_DIR" --profiles-dir "$DBT_PROJECT_DIR"

MANIFEST="$DBT_PROJECT_DIR/target/manifest.json"

# Validate columns
echo "Validating report columns..."
python3 -c "
import json, yaml, sys

with open('$MANIFEST') as f:
    manifest = json.load(f)
with open('$REPORTS_YML') as f:
    reports = yaml.safe_load(f)

model_columns = {}
for node in manifest['nodes'].values():
    if node.get('resource_type') == 'model' and node.get('columns'):
        model_columns[node['name']] = list(node['columns'].keys())

errors = []
for job in reports['report_jobs']:
    expected = job.get('expected_columns')
    if expected is None:
        continue
    table = job['source_table']
    cols = model_columns.get(table)
    if cols is None:
        errors.append(f\"{job['norm']}/{job['id']}: source_table '{table}' has no columns in dbt manifest\")
        continue
    if job.get('supports_as_of', False):
        cols = [c for c in cols if c != 'as_of_date']
    if cols != expected:
        errors.append(f\"{job['norm']}/{job['id']}: column mismatch for '{table}': expected {expected}, got {cols}\")

if errors:
    print('Column validation failed:', file=sys.stderr)
    for e in errors:
        print(f'  - {e}', file=sys.stderr)
    sys.exit(1)
print('Column validation passed.')
"
