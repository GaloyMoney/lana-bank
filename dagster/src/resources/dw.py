"""Cross-platform Data Warehouse resource.

Provides a unified interface for DW operations regardless of whether
the target is BigQuery or Postgres.
"""

import json
import os
from abc import ABC, abstractmethod
from typing import Any, Dict, Optional

import dagster as dg

RESOURCE_KEY_DW = "dw"


def get_dw_target() -> str:
    """Get the data warehouse target from environment."""
    return os.getenv("DW_TARGET", "bigquery").lower()


class BaseDWResource(ABC):
    """Abstract base class for DW resources."""

    @abstractmethod
    def get_raw_schema(self) -> str:
        """Get the schema where raw/source data is loaded."""
        pass

    @abstractmethod
    def get_dbt_schema(self) -> str:
        """Get the schema where dbt writes models."""
        pass

    @abstractmethod
    def get_credentials(self) -> Optional[Dict[str, Any]]:
        """Get credentials for dlt destination (if needed)."""
        pass


class BigQueryDWResource(dg.ConfigurableResource, BaseDWResource):
    """BigQuery Data Warehouse resource."""

    def get_credentials_json(self) -> str:
        """Get BigQuery credentials JSON from environment variable."""
        return dg.EnvVar("DBT_BIGQUERY_CREDENTIALS_JSON").get_value()

    def get_credentials(self) -> Dict[str, Any]:
        """Get BigQuery credentials as a dictionary."""
        return json.loads(self.get_credentials_json())

    def get_raw_schema(self) -> str:
        """Get the dataset where raw data is loaded."""
        return dg.EnvVar("DW_RAW_SCHEMA").get_value()

    def get_dbt_schema(self) -> str:
        """Get the dataset where dbt writes models."""
        return dg.EnvVar("DW_DBT_SCHEMA").get_value()

    # Legacy compatibility methods
    def get_target_dataset(self) -> str:
        """Legacy: Get raw data dataset."""
        return self.get_raw_schema()

    def get_dbt_dataset(self) -> str:
        """Legacy: Get dbt output dataset."""
        return self.get_dbt_schema()

    def get_credentials_dict(self) -> Dict[str, Any]:
        """Legacy: Alias for get_credentials."""
        return self.get_credentials()

    def get_client(self):
        """Create a BigQuery client from credentials."""
        from google.cloud import bigquery
        from google.oauth2 import service_account

        credentials_dict = self.get_credentials()
        creds = service_account.Credentials.from_service_account_info(credentials_dict)
        project_id = credentials_dict["project_id"]
        return bigquery.Client(project=project_id, credentials=creds)


class PostgresDWResource(dg.ConfigurableResource, BaseDWResource):
    """Postgres Data Warehouse resource."""

    def get_raw_schema(self) -> str:
        """Get the schema where raw data is loaded."""
        return os.getenv("DW_RAW_SCHEMA", "raw")

    def get_dbt_schema(self) -> str:
        """Get the schema where dbt writes models."""
        return os.getenv("DW_DBT_SCHEMA", "dbt")

    def get_credentials(self) -> Optional[Dict[str, Any]]:
        """Postgres credentials are handled by dlt directly from env vars."""
        return None

    def get_connection_params(self) -> Dict[str, Any]:
        """Get Postgres connection parameters."""
        return {
            "host": os.getenv("DW_PG_HOST", "localhost"),
            "port": int(os.getenv("DW_PG_PORT", "5432")),
            "database": os.getenv("DW_PG_DATABASE", "lana_dw"),
            "user": os.getenv("DW_PG_USER", "postgres"),
            "password": os.getenv("DW_PG_PASSWORD", ""),
        }

    def get_connection_string(self) -> str:
        """Get Postgres connection string."""
        params = self.get_connection_params()
        return (
            f"postgresql://{params['user']}:{params['password']}"
            f"@{params['host']}:{params['port']}/{params['database']}"
        )


def create_dw_resource() -> dg.ConfigurableResource:
    """Factory to create the appropriate DW resource based on DW_TARGET."""
    target = get_dw_target()
    
    if target == "bigquery":
        return BigQueryDWResource()
    elif target == "postgres":
        return PostgresDWResource()
    else:
        raise ValueError(f"Unknown DW_TARGET: {target}")
