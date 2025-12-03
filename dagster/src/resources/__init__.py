"""Dagster resources module - provides all resource classes and utilities."""

from src.resources.bigquery import RESOURCE_KEY_DW_BQ, BigQueryResource
from src.resources.dbt import DBT_MANIFEST_PATH, RESOURCE_KEY_LANA_DBT, dbt_resource
from src.resources.gcs import GCSResource, RESOURCE_KEY_FILE_REPORTS_BUCKET
from src.resources.postgres import PostgresResource, RESOURCE_KEY_LANA_CORE_PG

__all__ = [
    # Constants
    "RESOURCE_KEY_LANA_CORE_PG",
    "RESOURCE_KEY_DW_BQ",
    "RESOURCE_KEY_FILE_REPORTS_BUCKET",
    "RESOURCE_KEY_LANA_DBT",
    # Resource classes
    "PostgresResource",
    "BigQueryResource",
    "GCSResource",
    # DBT
    "dbt_resource",
    "DBT_MANIFEST_PATH",
    # Functions
    "get_project_resources",
]


def get_project_resources():
    """Get all project resources as a dictionary."""
    resources = {}
    resources[RESOURCE_KEY_LANA_CORE_PG] = PostgresResource()
    resources[RESOURCE_KEY_DW_BQ] = BigQueryResource()
    resources[RESOURCE_KEY_FILE_REPORTS_BUCKET] = GCSResource()
    resources[RESOURCE_KEY_LANA_DBT] = dbt_resource
    return resources

