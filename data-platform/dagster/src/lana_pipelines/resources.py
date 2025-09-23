from pathlib import Path

from dagster_dbt import DbtCliResource

from dlt.sources.sql_database import sql_table

def create_postgres_resource(connection_string_details, table_name):
    postgres_resource = sql_table(
        credentials=connection_string_details,
        schema="public",
        backend="sqlalchemy",
        table=table_name,
    )

    return postgres_resource

dbt_resource = DbtCliResource(project_dir=Path("dbt_lana_dw/"))
dbt_parse_invocation = dbt_resource.cli(["parse"], manifest={}).wait()
dbt_manifest_path = dbt_parse_invocation.target_path.joinpath("manifest.json")
