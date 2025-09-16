from dlt.sources.sql_database import sql_table


def create_postgres_resource(connection_string_details, table_name):
    postgres_resource = sql_table(
        credentials=connection_string_details,
        schema="public",
        backend="sqlalchemy",
        table=table_name,
    )

    return postgres_resource
