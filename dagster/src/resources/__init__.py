"""Dagster resources module - provides all resource classes and utilities."""

from src.resources.bigquery import RESOURCE_KEY_DW_BQ, BigQueryResource
from src.resources.dbt import DBT_MANIFEST_PATH, RESOURCE_KEY_LANA_DBT, dbt_resource
from src.resources.gcs import RESOURCE_KEY_FILE_REPORTS_BUCKET, GCSResource
from src.resources.postgres import RESOURCE_KEY_LANA_CORE_PG, PostgresResource
from src.resources.sumsub import RESOURCE_KEY_SUMSUB, SumsubResource

__all__ = [
    "RESOURCE_KEY_LANA_CORE_PG",
    "RESOURCE_KEY_DW_BQ",
    "RESOURCE_KEY_FILE_REPORTS_BUCKET",
    "RESOURCE_KEY_LANA_DBT",
    "RESOURCE_KEY_SUMSUB",
    "PostgresResource",
    "BigQueryResource",
    "GCSResource",
    "SumsubResource",
    "dbt_resource",
    "DBT_MANIFEST_PATH",
    "get_project_resources",
]


def get_project_resources():
    """Get all project resources as a dictionary."""
    resources = {}
    resources[RESOURCE_KEY_LANA_CORE_PG] = PostgresResource()
    resources[RESOURCE_KEY_DW_BQ] = BigQueryResource()
    resources[RESOURCE_KEY_FILE_REPORTS_BUCKET] = GCSResource()
    resources[RESOURCE_KEY_LANA_DBT] = dbt_resource
    resources[RESOURCE_KEY_SUMSUB] = SumsubResource()
    return resources
