from typing import Any

from dlt.sources.credentials import ConnectionStringCredentials
from dlt.sources.sql_database import sql_table

import dagster as dg
from src.dlt_destinations.bigquery import create_bigquery_destination


def create_sql_table_resource(crendetials: ConnectionStringCredentials, table_name):
    return sql_table(
        credentials=crendetials,
        schema="public",
        backend="sqlalchemy",
        table=table_name,
    )


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
        dlt_postgres_resource = create_sql_table_resource(
            crendetials=self.get_credentials(), table_name=table_name
        )

        return dlt_postgres_resource


class BigQueryResource(dg.ConfigurableResource):
    base64_credentials: Any
    target_dataset: str = "set_your_dataset"

    def get_dlt_destination(self):
        dlt_destination = create_bigquery_destination(self.base64_credentials)
        return dlt_destination
