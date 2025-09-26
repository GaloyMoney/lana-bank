from pathlib import Path

from dagster_dbt import DbtCliResource

from dlt.sources.sql_database import sql_table, sql_database

def create_postgres_resource(connection_string_details, table_name):
    postgres_resource = sql_table(
        credentials=connection_string_details,
        schema="public",
        backend="sqlalchemy",
        table=table_name,
    )

    return postgres_resource


def poll_max_value_in_table_col(connection_string_details, table_name, fieldname):
    
    import psycopg2

    conn = psycopg2.connect(connection_string_details)
    with conn, conn.cursor() as cur:
        cur.execute(f"SELECT MAX({fieldname}) FROM {table_name}")
        max_created_at = cur.fetchone()[0]
    
    return max_created_at

dbt_resource = DbtCliResource(project_dir=Path("dbt_lana_dw/"))
dbt_parse_invocation = dbt_resource.cli(["parse"], manifest={}).wait()
dbt_manifest_path = dbt_parse_invocation.target_path.joinpath("manifest.json")
