from typing import Dict

import dlt

import dagster as dg
from src.core import Protoasset
from src.dlt_destinations.bigquery import create_bigquery_destination
from src.dlt_resources.sumsub import applicants as dlt_sumsub_applicants
from src.resources import (
    RESOURCE_KEY_DW_BQ,
    RESOURCE_KEY_SUMSUB,
    BigQueryResource,
    SumsubResource,
)


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


def sumsub_protoasset() -> Protoasset:
    """Return the single Sumsub applicants protoasset."""
    return Protoasset(
        key=dg.AssetKey("sumsub_applicants"),
        callable=sumsub_applicants,
        required_resource_keys={
            RESOURCE_KEY_DW_BQ,
            RESOURCE_KEY_SUMSUB,
        },
        deps=[dg.AssetKey(["lana", "inbox_events"])],
        tags={"system": "sumsub", "asset_type": "el_target_asset"},
    )
