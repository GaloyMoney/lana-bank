import json

from google.cloud import storage
from google.oauth2 import service_account

import dagster as dg
from dagster_dbt import DbtCliResource

RESOURCE_KEY_FILE_REPORTS_BUCKET = "file_reports_bucket"
RESOURCE_KEY_LANA_DBT = "dbt"


class GCSResource(dg.ConfigurableResource):
    """Dagster resource for Google Cloud Storage."""

    def get_credentials_dict(self) -> dict:
        """Get GCS credentials dictionary from environment variable."""
        creds_json = dg.EnvVar("DBT_BIGQUERY_CREDENTIALS_JSON").get_value()
        if not creds_json:
            raise ValueError(
                "DBT_BIGQUERY_CREDENTIALS_JSON environment variable is not set or is empty. "
                "Ensure TF_VAR_sa_creds is set in .env and .envrc is loaded (via direnv allow) before starting containers."
            )
        return json.loads(creds_json)

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
