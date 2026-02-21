"""Dagster resources module - provides all resource classes and utilities."""

from src.resources.bigquery import RESOURCE_KEY_DW_BQ, BigQueryResource
from src.resources.dbt import DBT_MANIFEST_PATH, RESOURCE_KEY_LANA_DBT, dbt_resource
from src.resources.dw import (
    RESOURCE_KEY_DW,
    BaseDWResource,
    BigQueryDWResource,
    PostgresDWResource,
    create_dw_resource,
    get_dw_target,
)
from src.resources.gcs import RESOURCE_KEY_FILE_REPORTS_BUCKET, GCSResource
from src.resources.postgres import RESOURCE_KEY_LANA_CORE_PG, PostgresResource
from src.resources.sumsub import RESOURCE_KEY_SUMSUB, SumsubResource

__all__ = [
    # Core source
    "RESOURCE_KEY_LANA_CORE_PG",
    "PostgresResource",
    # Data warehouse (unified)
    "RESOURCE_KEY_DW",
    "BaseDWResource",
    "BigQueryDWResource",
    "PostgresDWResource",
    "create_dw_resource",
    "get_dw_target",
    # Legacy BigQuery (for backward compat)
    "RESOURCE_KEY_DW_BQ",
    "BigQueryResource",
    # Other resources
    "RESOURCE_KEY_FILE_REPORTS_BUCKET",
    "RESOURCE_KEY_LANA_DBT",
    "RESOURCE_KEY_SUMSUB",
    "GCSResource",
    "SumsubResource",
    "dbt_resource",
    "DBT_MANIFEST_PATH",
    "get_project_resources",
]


def get_project_resources():
    """Get all project resources as a dictionary.
    
    Resources are selected based on DW_TARGET environment variable:
    - 'bigquery' (default): Uses BigQuery for data warehouse
    - 'postgres': Uses Postgres for data warehouse
    """
    resources = {}
    
    # Source database (always Postgres - the lana-bank core)
    resources[RESOURCE_KEY_LANA_CORE_PG] = PostgresResource()
    
    # Data warehouse (depends on DW_TARGET)
    resources[RESOURCE_KEY_DW] = create_dw_resource()
    
    # Legacy: Also expose as dw_bq for backward compatibility when using BigQuery
    target = get_dw_target()
    if target == "bigquery":
        resources[RESOURCE_KEY_DW_BQ] = BigQueryResource()
    
    # Other resources
    resources[RESOURCE_KEY_FILE_REPORTS_BUCKET] = GCSResource()
    resources[RESOURCE_KEY_LANA_DBT] = dbt_resource
    resources[RESOURCE_KEY_SUMSUB] = SumsubResource()
    
    return resources
