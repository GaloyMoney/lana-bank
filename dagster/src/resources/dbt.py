import os
from pathlib import Path

from dagster_dbt import DbtCliResource

RESOURCE_KEY_LANA_DBT = "dbt"

DBT_PROJECT_DIR = Path("/lana-dw/src/dbt_lana_dw/")

# Manifest was pre-generated at Docker build time
DBT_MANIFEST_PATH = DBT_PROJECT_DIR / "target" / "manifest.json"

dbt_resource = DbtCliResource(project_dir=DBT_PROJECT_DIR)

# Only run UDF creation in the gRPC code server, not in worker subprocesses.
# DAGSTER_GRPC_HOST is set in the gRPC server but not in multiprocess workers.
# Set SKIP_BIGQUERY_UDFS=true in environments without BigQuery access (e.g., smoketests)
running_in_worker = os.environ.get("DAGSTER_GRPC_HOST") is None
should_skip_udfs = os.environ.get("SKIP_BIGQUERY_UDFS", "").lower() == "true"

if not running_in_worker and not should_skip_udfs:
    dbt_resource.cli(["run-operation", "create_udfs"]).wait()
