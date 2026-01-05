"""BigQuery table management utilities."""

from typing import List

from google.cloud import bigquery


def table_exists(client: bigquery.Client, dataset: str, table_name: str) -> bool:
    """Check if a BigQuery table exists."""
    table_id = f"{client.project}.{dataset}.{table_name}"
    try:
        client.get_table(table_id)
        return True
    except Exception:
        return False


def create_empty_table(
    client: bigquery.Client,
    dataset: str,
    table_name: str,
    schema: List[bigquery.SchemaField],
) -> None:
    """
    Create an empty BigQuery table with the given schema.
    """
    table_id = f"{client.project}.{dataset}.{table_name}"
    table = bigquery.Table(table_id, schema=schema)
    client.create_table(table)
