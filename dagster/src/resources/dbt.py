from pathlib import Path

from dagster_dbt import DbtCliResource

RESOURCE_KEY_LANA_DBT = "dbt"

DBT_PROJECT_DIR = Path("/lana-dw/src/dbt_lana_dw/")

# Manifest was pre-generated at Docker build time
DBT_MANIFEST_PATH = DBT_PROJECT_DIR / "target" / "manifest.json"

dbt_resource = DbtCliResource(project_dir=DBT_PROJECT_DIR)
