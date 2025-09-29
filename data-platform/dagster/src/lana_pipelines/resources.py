from pathlib import Path
from typing import Any

import dagster as dg
from dagster_dbt import DbtCliResource

from dlt.sources.credentials import ConnectionStringCredentials
from dlt.sources.sql_database import sql_table, sql_database
import psycopg2

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

    def create_dlt_postgres_resource(self, table_name):
        dlt_postgres_resource = sql_table(
            credentials=self.get_credentials(),
            schema="public",
            backend="sqlalchemy",
            table=table_name,
        )

        return dlt_postgres_resource
    
    def poll_max_value_in_table_col(self, table_name, fieldname):
        credentials = self.get_credentials()
        dsn = (
            f"dbname={credentials.database} "
            f"user={credentials.username} "
            f"password={credentials.password} "
            f"host={credentials.host} "
            f"port={credentials.port}"
        )
        conn = psycopg2.connect(dsn)
        with conn, conn.cursor() as cur:
            cur.execute(f"SELECT MAX({fieldname}) FROM {table_name}")
            max_created_at = cur.fetchone()[0]
        
        return max_created_at

class BigQueryResource (dg.ConfigurableResource):
    base64_credentials: Any

    def get_dlt_destination(self):
        dlt_destination = create_bigquery_destination(self.base64_credentials)
        return dlt_destination






dbt_resource = DbtCliResource(project_dir=Path("dbt_lana_dw/"))
dbt_parse_invocation = dbt_resource.cli(["parse"], manifest={}).wait()
dbt_manifest_path = dbt_parse_invocation.target_path.joinpath("manifest.json")
