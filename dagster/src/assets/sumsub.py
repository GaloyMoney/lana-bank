"""Sumsub DLT assets."""

import os
from typing import Dict

import dlt

import dagster as dg
from src.core import Protoasset
from src.dlt_destinations.bigquery import create_bigquery_destination
from src.dlt_resources.sumsub import applicants as dlt_sumsub_applicants
from src.resources import (
    RESOURCE_KEY_DW_BQ,
    RESOURCE_KEY_LANA_CORE_PG,
    BigQueryResource,
    PostgresResource,
)


def sumsub_applicants(
    context: dg.AssetExecutionContext,
    lana_core_pg: PostgresResource,
    dw_bq: BigQueryResource,
) -> None:
    """Runs the Sumsub applicants DLT pipeline into BigQuery."""
    sumsub_key = os.getenv("SUMSUB_KEY")
    sumsub_secret = os.getenv("SUMSUB_SECRET")
    if not sumsub_key or not sumsub_secret:
        raise RuntimeError(
            "Missing SUMSUB_KEY or SUMSUB_SECRET environment variables required to run Sumsub sync."
        )

    dest = create_bigquery_destination(dw_bq.get_credentials_dict())
    pipe = dlt.pipeline(
        pipeline_name="sumsub_applicants",
        destination=dest,
        dataset_name=dw_bq.get_target_dataset(),
    )

    dlt_resource = dlt_sumsub_applicants(
        pg_connection_string=lana_core_pg.get_connection_string(),
        sumsub_key=sumsub_key,
        sumsub_secret=sumsub_secret,
    )

    load_info = pipe.run(dlt_resource)
    context.log.info(str(load_info))


def sumsub_protoassets() -> Dict[str, Protoasset]:
    """Return all Sumsub protoassets keyed by asset key."""
    return {
        "sumsub_applicants": Protoasset(
            key=dg.AssetKey("sumsub_applicants"),
            callable=sumsub_applicants,
            required_resource_keys={RESOURCE_KEY_LANA_CORE_PG, RESOURCE_KEY_DW_BQ},
        ),
    }
