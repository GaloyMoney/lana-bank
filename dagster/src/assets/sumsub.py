import dlt
from google.cloud import bigquery

import dagster as dg
from src.core import Protoasset
from src.dlt_destinations.bigquery import create_bigquery_destination
from src.dlt_resources.sumsub import (
    SUMSUB_APPLICANTS_DLT_TABLE,
    applicants as dlt_sumsub_applicants,
)
from src.assets.lana import LANA_SYSTEM_NAME

SUMSUB_SYSTEM_NAME = "sumsub"
from src.resources import (
    RESOURCE_KEY_DW_BQ,
    RESOURCE_KEY_SUMSUB,
    BigQueryResource,
    SumsubResource,
)
from src.utils import create_empty_table, table_exists


def sumsub_applicants(
    context: dg.AssetExecutionContext,
    dw_bq: BigQueryResource,
    sumsub: SumsubResource,
) -> None:
    """Runs the Sumsub applicants DLT pipeline into BigQuery."""
    sumsub_key, sumsub_secret = sumsub.get_auth()

    dest = create_bigquery_destination(dw_bq.get_credentials_dict())
    pipe = dlt.pipeline(
        pipeline_name="sumsub_applicants",
        destination=dest,
        dataset_name=dw_bq.get_target_dataset(),
    )

    dlt_resource = dlt_sumsub_applicants(
        bq_credentials=dw_bq.get_credentials_dict(),
        bq_dataset=dw_bq.get_target_dataset(),
        sumsub_key=sumsub_key,
        sumsub_secret=sumsub_secret,
        logger=context.log,
    )

    load_info = pipe.run(dlt_resource)
    context.log.info(str(load_info))

    ensure_sumsub_table_exists(context, dw_bq)


SUMSUB_APPLICANTS_DLT_SCHEMA = [
    bigquery.SchemaField("customer_id", "STRING", mode="REQUIRED"),
    bigquery.SchemaField("recorded_at", "TIMESTAMP", mode="REQUIRED"),
    bigquery.SchemaField("content", "STRING", mode="NULLABLE"),
    bigquery.SchemaField("document_images", "JSON", mode="NULLABLE"),
    bigquery.SchemaField("_dlt_load_id", "FLOAT", mode="REQUIRED"),
    bigquery.SchemaField("_dlt_id", "STRING", mode="REQUIRED"),
]


def ensure_sumsub_table_exists(
    context: dg.AssetExecutionContext,
    dw_bq: BigQueryResource,
) -> None:
    """
    Ensure the sumsub_applicants_dlt table exists in BigQuery.

    If DLT didn't create it (due to empty source), create it manually.
    """
    bq_client = dw_bq.get_client()
    bq_dataset = dw_bq.get_target_dataset()

    if table_exists(bq_client, bq_dataset, SUMSUB_APPLICANTS_DLT_TABLE):
        return

    create_empty_table(
        client=bq_client,
        dataset=bq_dataset,
        table_name=SUMSUB_APPLICANTS_DLT_TABLE,
        schema=SUMSUB_APPLICANTS_DLT_SCHEMA,
    )
    context.log.info(
        f"Created empty table {SUMSUB_APPLICANTS_DLT_TABLE} in BigQuery."
    )


def sumsub_protoasset() -> Protoasset:
    """Return the single Sumsub applicants protoasset."""
    return Protoasset(
        key=dg.AssetKey([SUMSUB_SYSTEM_NAME, SUMSUB_APPLICANTS_DLT_TABLE]),
        callable=sumsub_applicants,
        required_resource_keys={
            RESOURCE_KEY_DW_BQ,
            RESOURCE_KEY_SUMSUB,
        },
        deps=[dg.AssetKey([LANA_SYSTEM_NAME, "inbox_events"])],
        tags={"system": SUMSUB_SYSTEM_NAME, "asset_type": "el_target_asset"},
    )
