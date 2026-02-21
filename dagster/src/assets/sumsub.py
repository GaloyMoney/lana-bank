from typing import Union

import dlt

import dagster as dg
from src.assets.lana import LANA_SYSTEM_NAME
from src.core import COLD_START_CONDITION, Protoasset
from src.dlt_destinations import create_dw_destination, get_dw_target
from src.dlt_resources.sumsub import (
    SUMSUB_APPLICANTS_DLT_TABLE,
)
from src.dlt_resources.sumsub import applicants as dlt_sumsub_applicants
from src.resources import (
    RESOURCE_KEY_DW,
    RESOURCE_KEY_SUMSUB,
    BigQueryDWResource,
    PostgresDWResource,
    SumsubResource,
)

SUMSUB_SYSTEM_NAME = "sumsub"


def sumsub_applicants(
    context: dg.AssetExecutionContext,
    dw: Union[BigQueryDWResource, PostgresDWResource],
    sumsub: SumsubResource,
) -> None:
    """Runs the Sumsub applicants DLT pipeline into the data warehouse."""
    sumsub_key, sumsub_secret = sumsub.get_auth()
    target = get_dw_target()

    dest = create_dw_destination(dw.get_credentials())
    raw_schema = dw.get_raw_schema()

    pipe = dlt.pipeline(
        pipeline_name="sumsub_applicants",
        destination=dest,
        dataset_name=raw_schema,
    )

    # For BigQuery, pass credentials to the dlt resource for incremental loading
    bq_credentials = dw.get_credentials() if target == "bigquery" else None
    
    dlt_resource = dlt_sumsub_applicants(
        bq_credentials=bq_credentials,
        bq_dataset=raw_schema if target == "bigquery" else None,
        sumsub_key=sumsub_key,
        sumsub_secret=sumsub_secret,
        logger=context.log,
    )

    load_info = pipe.run(dlt_resource)
    context.log.info(str(load_info))

    # Ensure table exists (BigQuery-specific for now)
    if target == "bigquery":
        ensure_sumsub_table_exists_bq(context, dw)


def ensure_sumsub_table_exists_bq(
    context: dg.AssetExecutionContext,
    dw: BigQueryDWResource,
) -> None:
    """
    Ensure the sumsub_applicants_dlt table exists in BigQuery.

    If DLT didn't create it (due to empty source), create it manually.
    """
    from google.cloud import bigquery
    from src.utils import create_empty_table, table_exists

    SUMSUB_APPLICANTS_DLT_SCHEMA = [
        bigquery.SchemaField("customer_id", "STRING", mode="REQUIRED"),
        bigquery.SchemaField("recorded_at", "TIMESTAMP", mode="REQUIRED"),
        bigquery.SchemaField("content", "STRING", mode="NULLABLE"),
        bigquery.SchemaField("document_images", "JSON", mode="NULLABLE"),
        bigquery.SchemaField("_dlt_load_id", "FLOAT", mode="REQUIRED"),
        bigquery.SchemaField("_dlt_id", "STRING", mode="REQUIRED"),
    ]

    bq_client = dw.get_client()
    raw_schema = dw.get_raw_schema()

    if table_exists(bq_client, raw_schema, SUMSUB_APPLICANTS_DLT_TABLE):
        return

    create_empty_table(
        client=bq_client,
        dataset=raw_schema,
        table_name=SUMSUB_APPLICANTS_DLT_TABLE,
        schema=SUMSUB_APPLICANTS_DLT_SCHEMA,
    )
    context.log.info(f"Created empty table {SUMSUB_APPLICANTS_DLT_TABLE} in BigQuery.")


def sumsub_protoasset() -> Protoasset:
    """Return the single Sumsub applicants protoasset."""
    return Protoasset(
        key=dg.AssetKey([SUMSUB_SYSTEM_NAME, SUMSUB_APPLICANTS_DLT_TABLE]),
        callable=sumsub_applicants,
        required_resource_keys={
            RESOURCE_KEY_DW,
            RESOURCE_KEY_SUMSUB,
        },
        deps=[dg.AssetKey([LANA_SYSTEM_NAME, "inbox_events"])],
        tags={"system": SUMSUB_SYSTEM_NAME, "asset_type": "el_target_asset"},
        automation_condition=COLD_START_CONDITION,
    )
