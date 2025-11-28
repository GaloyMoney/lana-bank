import base64
import json
from pathlib import Path
from typing import Any

from dagster_dbt import DbtCliResource
from google.cloud import storage
from google.oauth2 import service_account

import dagster as dg

RESOURCE_KEY_LANA_CORE_PG = "lana_core_pg"
RESOURCE_KEY_DW_BQ = "dw_bq"
RESOURCE_KEY_FILE_REPORTS_BUCKET = "file_reports_bucket"
RESOURCE_KEY_LANA_DBT = "dbt"


class PostgresResource(dg.ConfigurableResource):
    """Dagster resource for PostgreSQL connection configuration."""

    def get_connection_string(self) -> str:
        return dg.EnvVar("LANA_PG_CON").get_value()


class BigQueryResource(dg.ConfigurableResource):
    """Dagster resource for BigQuery configuration."""

    def get_base64_credentials(self) -> str:
        return dg.EnvVar("TF_VAR_sa_creds").get_value()

    def get_target_dataset(self) -> str:
        return dg.EnvVar("TARGET_BIGQUERY_DATASET").get_value()


class GCSResource(dg.ConfigurableResource):
    """Dagster resource for Google Cloud Storage."""

    def get_credentials_dict(self) -> dict:
        """Get GCS credentials dictionary from environment variable."""
        base64_creds = dg.EnvVar("TF_VAR_sa_creds").get_value()
        creds_json = base64.b64decode(base64_creds).decode("utf-8")
        creds_dict = json.loads(creds_json)
        return creds_dict

    def get_credentials(self) -> service_account.Credentials:
        """Get GCS credentials from environment variable."""
        creds_dict = self.get_credentials_dict()
        return service_account.Credentials.from_service_account_info(creds_dict)

    def get_project_id(self) -> str:
        """Get GCP project ID from credentials."""
        creds_dict = self.get_credentials_dict()
        return creds_dict.get("project_id", "")

    def get_bucket_name(self) -> str:
        """Get GCS bucket name from environment variable."""
        bucket_name = dg.EnvVar("REPORTS_BUCKET_NAME").get_value()
        if not bucket_name:
            raise ValueError(
                "REPORTS_BUCKET_NAME environment variable is not set or is empty. "
                "Please set it in your .envrc or environment configuration."
            )
        return bucket_name

    def get_client(self) -> storage.Client:
        """Get a GCS client."""
        credentials = self.get_credentials()
        project_id = self.get_project_id()
        if not project_id:
            raise ValueError(
                "Could not extract project_id from service account credentials."
            )
        return storage.Client(project=project_id, credentials=credentials)

    def upload_file(self, content: bytes, path: str, content_type: str) -> str:
        """Upload a file to GCS and return the GCS path."""
        client = self.get_client()
        bucket_name = self.get_bucket_name()

        # Get the bucket - this will fail if bucket doesn't exist
        bucket = client.bucket(bucket_name)

        # Ensure bucket exists (or create it)
        if not bucket.exists():
            raise ValueError(
                f"GCS bucket '{bucket_name}' does not exist. "
                f"Please create the bucket first or update the bucket name."
            )

        blob = bucket.blob(path)
        blob.upload_from_string(content, content_type=content_type)
        return f"gs://{bucket_name}/{path}"


dbt_resource = DbtCliResource(project_dir=Path("/lana-dw/src/dbt_lana_dw/"))
dbt_parse_invocation = dbt_resource.cli(["parse"], manifest={}).wait()
dbt_manifest_path = dbt_parse_invocation.target_path.joinpath("manifest.json")


def get_project_resources():
    resources = {}
    resources[RESOURCE_KEY_LANA_CORE_PG] = PostgresResource()
    resources[RESOURCE_KEY_DW_BQ] = BigQueryResource()
    resources[RESOURCE_KEY_FILE_REPORTS_BUCKET] = GCSResource()
    resources[RESOURCE_KEY_LANA_DBT] = dbt_resource
    return resources
