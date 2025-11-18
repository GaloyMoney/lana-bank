from typing import List

import dlt

import dagster as dg
from src.core import Protoasset
from src.dlt_sources.resources import (
    BigQueryResource,
    PostgresResource,
)

LANA_EL_TABLE_NAMES = (
    "core_chart_events_rollup",
    "core_collateral_events_rollup",
    "core_credit_facility_events_rollup",
    "core_credit_facility_proposal_events_rollup",
    "core_customer_events_rollup",
    "core_deposit_account_events_rollup",
    "core_deposit_events_rollup",
    "core_disbursal_events_rollup",
    "core_interest_accrual_cycle_events_rollup",
    "core_liquidation_process_events_rollup",
    "core_obligation_events_rollup",
    "core_payment_allocation_events_rollup",
    "core_payment_events_rollup",
    "core_pending_credit_facility_events_rollup",
    "core_withdrawal_events_rollup",
    "core_public_ids",
    "core_chart_events",
    "core_chart_node_events",
)


def lana_resources():
    resources = {}
    resources["lana_core_pg"] = PostgresResource()
    resources["dw_bq"] = BigQueryResource(
        base64_credentials=dg.EnvVar("TF_VAR_sa_creds").get_value(),
        target_dataset=dg.EnvVar("TARGET_BIGQUERY_DATASET").get_value(),
    )
    return resources


def lana_source_protoassets() -> List[Protoasset]:
    lana_source_protoassets = []
    for table_name in LANA_EL_TABLE_NAMES:
        lana_source_protoassets.append(
            Protoasset(
                key=f"el_source_asset__lana__{table_name}",
                tags={"asset_type": "el_source__asset", "system": "lana"},
            )
        )
    return lana_source_protoassets


def lana_to_dw_el_protoassets() -> List[Protoasset]:
    lana_el_protoassets = []
    for table_name in LANA_EL_TABLE_NAMES:
        lana_el_protoassets.append(
            build_lana_to_dw_el_protoasset(
                table_name=table_name,
            )
        )

    return lana_el_protoassets


def build_lana_to_dw_el_protoasset(table_name) -> Protoasset:

    def lana_to_dw_el_asset(
        context: dg.AssetExecutionContext,
        lana_core_pg: PostgresResource,
        dw_bq: BigQueryResource,
    ):
        context.log.info(
            f"Running lana_to_dw_el_asset pipeline for table {table_name}."
        )

        runnable_pipeline = prepare_lana_el_pipeline(
            lana_core_pg=lana_core_pg, dw_bq=dw_bq, table_name=table_name
        )
        load_info = runnable_pipeline()

        context.log.info(f"Pipeline completed.")
        context.log.info(load_info)
        return load_info

    lana_to_dw_protoasset = Protoasset(
        key=["lana", table_name],
        deps=[f"el_source_asset__lana__{table_name}"],
        tags={"asset_type": "el_target__asset", "system": "lana"},
        callable=lana_to_dw_el_asset,
    )

    return lana_to_dw_protoasset


def prepare_lana_el_pipeline(lana_core_pg, dw_bq, table_name):
    dlt_postgres_resource = lana_core_pg.create_dlt_postgres_resource(
        table_name=table_name
    )
    dlt_bq_destination = dw_bq.get_dlt_destination()

    pipeline = dlt.pipeline(
        pipeline_name=table_name,
        destination=dlt_bq_destination,
        dataset_name=dw_bq.target_dataset,
    )

    # Ready to be called with source and disposition already hardcoded
    def wrapped_pipeline():
        load_info = pipeline.run(
            dlt_postgres_resource,
            write_disposition="replace",
            table_name=table_name,
        )
        return load_info

    return wrapped_pipeline
