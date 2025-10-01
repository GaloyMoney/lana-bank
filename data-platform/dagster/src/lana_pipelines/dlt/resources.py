from dlt.sources.sql_database import sql_table
from dlt.sources.credentials import ConnectionStringCredentials


def create_sql_table_resource(crendetials: ConnectionStringCredentials, table_name):
    return sql_table(
        credentials=crendetials,
        schema="public",
        backend="sqlalchemy",
        table=table_name,
    )
