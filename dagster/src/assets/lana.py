from typing import List

import dlt

import dagster as dg
from src.core import Protoasset
from src.dlt_destinations.bigquery import create_bigquery_destination
from src.dlt_resources.postgres import create_dlt_postgres_resource
from src.resources import (
    RESOURCE_KEY_DW_BQ,
    RESOURCE_KEY_LANA_CORE_PG,
    BigQueryResource,
    PostgresResource,
)
from src.utils import (
    create_empty_table,
    get_postgres_table_schema,
    postgres_schema_to_bigquery_schema,
    table_exists,
)

LANA_EL_TABLE_NAMES = (
    "core_chart_events_rollup",
    "core_credit_facility_events_rollup",
    "core_credit_facility_proposal_events_rollup",
    "core_customer_events_rollup",
    "core_deposit_account_events_rollup",
    "core_deposit_events_rollup",
    "core_disbursal_events_rollup",
    "core_interest_accrual_cycle_events_rollup",
    "core_obligation_events_rollup",
    "core_payment_allocation_events_rollup",
    "core_payment_events_rollup",
    "core_pending_credit_facility_events_rollup",
    "core_withdrawal_events_rollup",
    "core_public_ids",
    "core_chart_events",
    "core_chart_node_events",
    "cala_account_set_member_account_sets",
    "cala_account_set_member_accounts",
    "cala_account_sets",
    "cala_accounts",
    "cala_balance_history",
    "inbox_events",
)

EL_SOURCE_ASSET_DESCRIPTION = "el_source_asset"
EL_TARGET_ASSET_DESCRIPTION = "el_target_asset"
LANA_SYSTEM_NAME = "lana"


def get_el_source_asset_name(system_name: str, table_name: str) -> str:
    return f"{EL_SOURCE_ASSET_DESCRIPTION}__{system_name}__{table_name}"


def lana_source_protoassets() -> List[Protoasset]:
    lana_source_protoassets = []
    for table_name in LANA_EL_TABLE_NAMES:
        lana_source_protoassets.append(
            Protoasset(
                key=dg.AssetKey(
                    get_el_source_asset_name(
                        system_name=LANA_SYSTEM_NAME, table_name=table_name
                    )
                ),
                tags={
                    "asset_type": EL_SOURCE_ASSET_DESCRIPTION,
                    "system": LANA_SYSTEM_NAME,
                },
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

        context.log.info("Pipeline completed.")
        context.log.info(load_info)

        # Why wouldn't a table exist?
        # Because if the source table has no data, dlt won't even create the
        # destination table.
        ensure_target_table_exists(
            context=context,
            lana_core_pg=lana_core_pg,
            dw_bq=dw_bq,
            table_name=table_name,
        )

        return load_info

    lana_to_dw_protoasset = Protoasset(
        key=dg.AssetKey([LANA_SYSTEM_NAME, table_name]),
        deps=[
            dg.AssetKey(
                get_el_source_asset_name(
                    system_name=LANA_SYSTEM_NAME, table_name=table_name
                )
            )
        ],
        tags={"asset_type": EL_TARGET_ASSET_DESCRIPTION, "system": LANA_SYSTEM_NAME},
        callable=lana_to_dw_el_asset,
        required_resource_keys={RESOURCE_KEY_LANA_CORE_PG, RESOURCE_KEY_DW_BQ},
    )

    return lana_to_dw_protoasset


def ensure_target_table_exists(
    context: dg.AssetExecutionContext,
    lana_core_pg: PostgresResource,
    dw_bq: BigQueryResource,
    table_name: str,
) -> None:
    """
    Ensure the target BigQuery table exists.

    If the table doesn't exist (because DLT didn't create it due to empty source),
    create it with schema inferred from the Postgres source table.
    """
    bq_dataset = dw_bq.get_target_dataset()
    bq_client = dw_bq.get_client()

    if table_exists(bq_client, bq_dataset, table_name):
        context.log.info(f"Target table {table_name} already exists in BigQuery.")
        return

    context.log.info(
        f"Target table {table_name} does not exist. Creating from Postgres schema..."
    )

    pg_columns = get_postgres_table_schema(
        connection_string=lana_core_pg.get_connection_string(),
        table_name=table_name,
    )

    if not pg_columns:
        context.log.warning(
            f"Could not get schema for table {table_name} from Postgres."
        )
        return

    bq_schema = postgres_schema_to_bigquery_schema(pg_columns)

    create_empty_table(
        client=bq_client,
        dataset=bq_dataset,
        table_name=table_name,
        schema=bq_schema,
    )

    context.log.info(
        f"Created empty table {table_name} in BigQuery with {len(bq_schema)} columns."
    )


def prepare_lana_el_pipeline(lana_core_pg, dw_bq, table_name):
    dlt_postgres_resource = create_dlt_postgres_resource(
        connection_string=lana_core_pg.get_connection_string(), table_name=table_name
    )
    dlt_bq_destination = create_bigquery_destination(dw_bq.get_credentials_dict())

    pipeline = dlt.pipeline(
        pipeline_name=table_name,
        destination=dlt_bq_destination,
        dataset_name=dw_bq.get_target_dataset(),
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
