from pathlib import Path

from dagster_dbt import DbtCliResource

RESOURCE_KEY_LANA_DBT = "dbt"

dbt_resource = DbtCliResource(project_dir=Path("/lana-dw/src/dbt_lana_dw/"))
dbt_parse_invocation = dbt_resource.cli(["parse"], manifest={}).wait()
DBT_MANIFEST_PATH = dbt_parse_invocation.target_path.joinpath("manifest.json")

# Run UDF creation once at startup (instead of on-run-start hook on every model run)
dbt_resource.cli(["run-operation", "create_udfs"]).wait()
