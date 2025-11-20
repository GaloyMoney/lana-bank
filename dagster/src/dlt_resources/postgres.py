"""DLT-specific PostgreSQL source creation utilities."""

from dlt.sources.credentials import ConnectionStringCredentials
from dlt.sources.sql_database import sql_table


def create_dlt_postgres_resource(connection_string: str, table_name: str) -> sql_table:
    """Create a DLT PostgreSQL table resource from a connection string."""
    credentials = ConnectionStringCredentials(connection_string)
    credentials.drivername = "postgresql"

    return sql_table(
        credentials=credentials,
        schema="public",
        backend="sqlalchemy",
        table=table_name,
    )
