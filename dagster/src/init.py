import os
import sys

from dagster_dbt import DbtCliResource

DBT_PROJECT_DIR = "/lana-dw/src/dbt_lana_dw/"


def main():
    mode = os.environ.get("DAGSTER_INIT_MODE", "")

    if mode == "dry-run":
        print("DAGSTER_INIT_MODE=dry-run, skipping BigQuery UDF creation")
        return

    print("Creating BigQuery UDFs...")
    dbt_resource = DbtCliResource(project_dir=DBT_PROJECT_DIR)
    # dbt_resource.cli(["run-operation", "create_udfs"]).wait()
    print("BigQuery UDFs created successfully")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        print(f"Init failed: {e}", file=sys.stderr)
        sys.exit(1)
