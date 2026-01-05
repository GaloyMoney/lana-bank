"""Postgres schema introspection utilities."""

from typing import Any, Dict, List

from sqlalchemy import create_engine, text
from sqlalchemy.engine import make_url


def get_postgres_table_schema(
    connection_string: str, table_name: str, schema: str = "public"
) -> List[Dict[str, Any]]:
    """
    Query the Postgres information_schema to get column definitions.

    Args:
        connection_string: PostgreSQL connection string
        table_name: Name of the table to inspect
        schema: Database schema (default: "public")

    Returns:
        A list of dicts with keys:
        - column_name: str
        - data_type: str (postgres type)
        - udt_name: str (underlying type name, useful for arrays)
        - is_nullable: bool
        - column_default: str or None
        - ordinal_position: int
    """
    url = make_url(connection_string)
    url = url.set(drivername="postgresql")
    engine = create_engine(url)

    query = text("""
        SELECT
            column_name,
            data_type,
            is_nullable,
            column_default,
            ordinal_position,
            udt_name
        FROM information_schema.columns
        WHERE table_schema = :schema AND table_name = :table_name
        ORDER BY ordinal_position
    """)

    with engine.connect() as conn:
        result = conn.execute(query, {"schema": schema, "table_name": table_name})
        columns = []
        for row in result:
            columns.append(
                {
                    "column_name": row.column_name,
                    "data_type": row.data_type,
                    "udt_name": row.udt_name,
                    "is_nullable": row.is_nullable == "YES",
                    "column_default": row.column_default,
                    "ordinal_position": row.ordinal_position,
                }
            )
        return columns
