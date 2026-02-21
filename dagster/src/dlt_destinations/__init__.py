"""DLT destination factory for cross-platform data loading."""

import os
from typing import Any, Dict

import dlt

from src.dlt_destinations.bigquery import create_bigquery_destination


def get_dw_target() -> str:
    """Get the data warehouse target from environment.
    
    Returns:
        Target name: 'bigquery' or 'postgres'
    """
    return os.getenv("DW_TARGET", "bigquery").lower()


def get_raw_schema() -> str:
    """Get the raw data schema name.
    
    This is where dlt loads source data (e.g., rollup tables, external APIs).
    """
    return os.getenv("DW_RAW_SCHEMA", "raw")


def create_dw_destination(credentials: Dict[str, Any] | None = None):
    """Create a dlt destination based on DW_TARGET environment variable.
    
    Args:
        credentials: Optional credentials dict. For BigQuery, this should be
                     the service account JSON. For Postgres, credentials are
                     read from environment variables.
    
    Returns:
        A dlt destination configured for the target warehouse.
    
    Raises:
        ValueError: If DW_TARGET is not a supported value.
    """
    target = get_dw_target()
    raw_schema = get_raw_schema()
    
    if target == "bigquery":
        if credentials is None:
            raise ValueError("BigQuery destination requires credentials")
        return create_bigquery_destination(credentials)
    
    elif target == "postgres":
        return dlt.destinations.postgres(
            credentials={
                "host": os.getenv("DW_PG_HOST", "localhost"),
                "port": int(os.getenv("DW_PG_PORT", "5432")),
                "database": os.getenv("DW_PG_DATABASE", "lana_dw"),
                "username": os.getenv("DW_PG_USER", "postgres"),
                "password": os.getenv("DW_PG_PASSWORD", ""),
            },
        )
    
    else:
        raise ValueError(
            f"Unknown DW_TARGET: '{target}'. "
            f"Supported values: 'bigquery', 'postgres'"
        )


__all__ = [
    "get_dw_target",
    "get_raw_schema", 
    "create_dw_destination",
    "create_bigquery_destination",
]
