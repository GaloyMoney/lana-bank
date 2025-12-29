import json
from typing import Optional

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

    def get_staging_bucket(self) -> Optional[str]:
        """Get GCS bucket name for dlt staging. Uses REPORTS_BUCKET_NAME."""
        return dg.EnvVar("REPORTS_BUCKET_NAME").get_value()
