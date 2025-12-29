from typing import Optional

import dlt


def create_bigquery_destination(
    credentials: dict, staging_bucket: Optional[str] = None
) -> dlt.destinations.bigquery:
    """Create a BigQuery destination for dlt.

    Args:
        credentials: Service account credentials as a dictionary.
        staging_bucket: Optional GCS bucket name for staging. If provided, data will be
                       staged to GCS before loading to BigQuery, which avoids rate limits
                       on concurrent table update operations.
    """
    required_fields = ["type", "project_id", "private_key", "client_email"]
    for field in required_fields:
        if field not in credentials:
            raise ValueError(f"Missing required field '{field}' in credentials")

    # Configure GCS staging if bucket is provided
    # This avoids BigQuery rate limits when running many parallel dlt pipelines,
    # as data is first written to GCS then loaded via batch load jobs.
    staging = None
    if staging_bucket:
        staging = dlt.destinations.filesystem(
            bucket_url=f"gs://{staging_bucket}/dlt_staging",
            credentials=credentials,
        )

    return dlt.destinations.bigquery(
        credentials=credentials,
        project_id=credentials["project_id"],
        location="US",
        staging=staging,
    )
