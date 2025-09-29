from pathlib import Path
from typing import Any

import dagster as dg
from dagster_dbt import DbtCliResource

from dlt.sources.credentials import ConnectionStringCredentials
from dlt.sources.sql_database import sql_table, sql_database

from lana_pipelines.destinations import create_bigquery_destination

class PostgresResource(dg.ConfigurableResource):

    def get_credentials(self):
        credentials = ConnectionStringCredentials()
        credentials.drivername = "postgresql"
        credentials.database = "pg"
        credentials.username = "user"
        credentials.password = "password"
        credentials.host = "172.17.0.1"
        credentials.port = 5433

        return credentials

class BigQueryResource (dg.ConfigurableResource):
    base64_credentials: Any
    
    def get_dlt_destination(self):
        dlt_destination = create_bigquery_destination(self.base64_credentials)
        return dlt_destination

def create_dlt_postgres_resource(connection_string_details, table_name):
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
