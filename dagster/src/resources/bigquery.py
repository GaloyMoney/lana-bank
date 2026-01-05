import json

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

    def get_target_dataset(self) -> str:
        return dg.EnvVar("TARGET_BIGQUERY_DATASET").get_value()

    def get_client(self) -> bigquery.Client:
        """Create a BigQuery client from credentials."""
        credentials_dict = self.get_credentials_dict()
        creds = service_account.Credentials.from_service_account_info(credentials_dict)
        project_id = credentials_dict["project_id"]
        return bigquery.Client(project=project_id, credentials=creds)
