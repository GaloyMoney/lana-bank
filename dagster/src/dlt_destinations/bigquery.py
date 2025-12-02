import dlt


def create_bigquery_destination(credentials: dict) -> dlt.destinations.bigquery:
    """Create a BigQuery destination for dlt.
    
    Args:
        credentials: Service account credentials as a dictionary.
    """
    required_fields = ["type", "project_id", "private_key", "client_email"]
    for field in required_fields:
        if field not in credentials:
            raise ValueError(f"Missing required field '{field}' in credentials")

    return dlt.destinations.bigquery(
        credentials=credentials,
        project_id=credentials["project_id"],
        location="US",
    )
