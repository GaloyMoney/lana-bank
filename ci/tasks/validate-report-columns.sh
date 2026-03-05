#!/bin/bash
set -euo pipefail

# Validate that reports declaring supports_as_of have an as_of_date column
# in their dbt source table.
#
# Generates the dbt manifest with dummy credentials (no BQ connection needed),
# then checks each as-of report against the manifest.

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

# Validate as-of reports have as_of_date column
echo "Validating as-of report columns..."
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
    if not job.get('supports_as_of', False):
        continue
    table = job['source_table']
    cols = model_columns.get(table)
    if cols is None:
        errors.append(f\"{job['norm']}/{job['id']}: source_table '{table}' has no columns declared in dbt manifest\")
    elif 'as_of_date' not in cols:
        errors.append(f\"{job['norm']}/{job['id']}: source_table '{table}' declares supports_as_of but has no as_of_date column (has: {cols})\")

if errors:
    print('Validation failed:', file=sys.stderr)
    for e in errors:
        print(f'  - {e}', file=sys.stderr)
    sys.exit(1)
print('As-of report validation passed.')
"
