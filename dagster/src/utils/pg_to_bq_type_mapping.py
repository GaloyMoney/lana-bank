"""Postgres to BigQuery type mapping utilities."""

from typing import Any, Dict, List, Tuple

from google.cloud import bigquery

POSTGRES_TO_BIGQUERY_TYPE_MAP = {
    # Numeric types
    "smallint": "INTEGER",
    "integer": "INTEGER",
    "int": "INTEGER",
    "int2": "INTEGER",
    "int4": "INTEGER",
    "bigint": "INTEGER",
    "int8": "INTEGER",
    "decimal": "BIGNUMERIC",
    "numeric": "BIGNUMERIC",
    "real": "FLOAT",
    "float4": "FLOAT",
    "double precision": "FLOAT",
    "float8": "FLOAT",
    # String types
    "character varying": "STRING",
    "varchar": "STRING",
    "character": "STRING",
    "char": "STRING",
    "text": "STRING",
    "citext": "STRING",
    # Boolean
    "boolean": "BOOLEAN",
    "bool": "BOOLEAN",
    # Date/Time types
    "date": "DATE",
    "timestamp without time zone": "TIMESTAMP",
    "timestamp with time zone": "TIMESTAMP",
    "timestamp": "TIMESTAMP",
    "timestamptz": "TIMESTAMP",
    "time without time zone": "TIME",
    "time with time zone": "TIME",
    "time": "TIME",
    "interval": "STRING",  # BQ doesn't have interval, store as string
    # Binary types
    "bytea": "BYTES",
    "bit": "BYTES",
    "varbit": "BYTES",
    "bit varying": "BYTES",
    # UUID
    "uuid": "STRING",
    # JSON types
    "json": "JSON",
    "jsonb": "JSON",
    "hstore": "JSON",
    # Network types
    "inet": "STRING",
    "cidr": "STRING",
    "macaddr": "STRING",
    "macaddr8": "STRING",
    # Geometric/Geographic types
    "geography": "GEOGRAPHY",
    "geometry": "GEOGRAPHY",
}

POSTGRES_ARRAY_ELEMENT_TO_BIGQUERY_MAP = {
    # Integer arrays
    "_int2": "INT64",
    "_int4": "INT64",
    "_int8": "INT64",
    "_integer": "INT64",
    # Float arrays
    "_float4": "FLOAT64",
    "_float8": "FLOAT64",
    "_double precision": "FLOAT64",
    # Boolean arrays
    "_bool": "BOOL",
    "_boolean": "BOOL",
    # String arrays
    "_varchar": "STRING",
    "_text": "STRING",
    "_char": "STRING",
    "_character varying": "STRING",
    # Date/Time arrays
    "_date": "DATE",
    "_timestamp": "TIMESTAMP",
    "_timestamptz": "TIMESTAMP",
    "_timestamp without time zone": "TIMESTAMP",
    "_timestamp with time zone": "TIMESTAMP",
}


def postgres_type_to_bigquery_type(
    pg_type: str, udt_name: str | None = None
) -> Tuple[str, bool]:
    """
    Convert a Postgres data type to a BigQuery data type.

    Args:
        pg_type: The data_type from information_schema.columns
        udt_name: The udt_name for more specific type info (e.g., _int4 for arrays)

    Returns:
        Tuple of (BigQuery type string, is_array boolean)
    """
    pg_type_lower = pg_type.lower()

    if pg_type_lower == "array" and udt_name:
        udt_lower = udt_name.lower()
        if udt_lower in POSTGRES_ARRAY_ELEMENT_TO_BIGQUERY_MAP:
            return (POSTGRES_ARRAY_ELEMENT_TO_BIGQUERY_MAP[udt_lower], True)
        return ("STRING", True)

    if pg_type_lower == "user-defined":
        return ("STRING", False)

    if pg_type_lower in POSTGRES_TO_BIGQUERY_TYPE_MAP:
        return (POSTGRES_TO_BIGQUERY_TYPE_MAP[pg_type_lower], False)

    if udt_name and udt_name.lower() in POSTGRES_TO_BIGQUERY_TYPE_MAP:
        return (POSTGRES_TO_BIGQUERY_TYPE_MAP[udt_name.lower()], False)

    return ("STRING", False)


def postgres_schema_to_bigquery_schema(
    pg_columns: List[Dict[str, Any]],
) -> List[bigquery.SchemaField]:
    """
    Convert a list of Postgres column definitions to BigQuery SchemaFields.

    Also adds the DLT metadata columns (_dlt_load_id, _dlt_id) that DLT would normally add.

    Array types in Postgres are mapped to REPEATED mode in BigQuery:
    - ARRAY<INT2>, ARRAY<INT>, ARRAY<INT8> -> ARRAY<INT64> (REPEATED INT64)
    - ARRAY<FLOAT4> -> ARRAY<FLOAT64> (REPEATED FLOAT64)
    - ARRAY<DOUBLE PRECISION> -> ARRAY<FLOAT64> (REPEATED FLOAT64)
    - ARRAY<BOOL> -> ARRAY<BOOL> (REPEATED BOOL)
    - ARRAY<VARCHAR>, ARRAY<TEXT> -> ARRAY<STRING> (REPEATED STRING)
    - ARRAY<DATE> -> ARRAY<DATE> (REPEATED DATE)
    - ARRAY<TIMESTAMP>, ARRAY<TIMESTAMPTZ> -> ARRAY<TIMESTAMP> (REPEATED TIMESTAMP)
    """
    bq_schema = []

    for col in pg_columns:
        bq_type, is_array = postgres_type_to_bigquery_type(
            col["data_type"], col.get("udt_name")
        )

        if is_array:
            mode = "REPEATED"
        elif col["is_nullable"]:
            mode = "NULLABLE"
        else:
            mode = "REQUIRED"

        bq_schema.append(
            bigquery.SchemaField(
                name=col["column_name"],
                field_type=bq_type,
                mode=mode,
            )
        )

    bq_schema.append(bigquery.SchemaField("_dlt_load_id", "STRING", mode="REQUIRED"))
    bq_schema.append(bigquery.SchemaField("_dlt_id", "STRING", mode="REQUIRED"))

    return bq_schema

