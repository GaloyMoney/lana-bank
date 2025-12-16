import json
from typing import Any

from google.cloud import bigquery
from google.oauth2 import service_account

import dagster as dg

RESOURCE_KEY_DW_BQ = "dw_bq"


class BigQueryResource(dg.ConfigurableResource):
    """Dagster resource for BigQuery configuration."""

    def get_credentials_json(self) -> str:
        """Get BigQuery credentials JSON from environment variable."""
        return dg.EnvVar("DBT_BIGQUERY_CREDENTIALS_JSON").get_value()

    def get_credentials_dict(self) -> dict:
        """Get BigQuery credentials as a dictionary."""
        return json.loads(self.get_credentials_json())

    def get_credentials(self) -> service_account.Credentials:
        """Get BigQuery credentials object."""
        return service_account.Credentials.from_service_account_info(
            self.get_credentials_dict()
        )

    def get_project_id(self) -> str:
        """Get GCP project ID from credentials."""
        return self.get_credentials_dict().get("project_id", "")

    def get_target_dataset(self) -> str:
        return dg.EnvVar("TARGET_BIGQUERY_DATASET").get_value()

    def get_client(self) -> bigquery.Client:
        """Get a BigQuery client."""
        return bigquery.Client(
            project=self.get_project_id(),
            credentials=self.get_credentials(),
        )

    def fetch_table(self, table_name: str) -> tuple[list[str], list[dict[str, Any]]]:
        """Fetch all rows from a table in the target dataset.

        Args:
            table_name: Name of the table to fetch.

        Returns:
            Tuple of (field_names, records) where records is a list of dicts.
        """
        client = self.get_client()
        project_id = self.get_project_id()
        dataset = self.get_target_dataset()

        query = f"SELECT * FROM `{project_id}.{dataset}.{table_name}`"
        query_job = client.query(query)
        rows = query_job.result()

        field_names = [field.name for field in rows.schema]
        records = [{name: row[name] for name in field_names} for row in rows]

        return field_names, records
