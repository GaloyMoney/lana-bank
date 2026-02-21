from typing import List, Union

import dlt

import dagster as dg
from src.core import COLD_START_CONDITION_SKIP_DEPS, Protoasset
from src.dlt_destinations import create_dw_destination, get_dw_target
from src.dlt_resources.postgres import create_dlt_postgres_resource
from src.resources import (
    RESOURCE_KEY_DW,
    RESOURCE_KEY_LANA_CORE_PG,
    BigQueryDWResource,
    PostgresDWResource,
    PostgresResource,
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
        dw: Union[BigQueryDWResource, PostgresDWResource],
    ):
        context.log.info(
            f"Running lana_to_dw_el_asset pipeline for table {table_name}."
        )

        runnable_pipeline = prepare_lana_el_pipeline(
            lana_core_pg=lana_core_pg, dw=dw, table_name=table_name
        )
        load_info = runnable_pipeline()

        context.log.info("Pipeline completed.")
        context.log.info(load_info)

        # Ensure target table exists (for BigQuery, create if empty source)
        target = get_dw_target()
        if target == "bigquery":
            ensure_target_table_exists_bq(
                context=context,
                lana_core_pg=lana_core_pg,
                dw=dw,
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
        required_resource_keys={RESOURCE_KEY_LANA_CORE_PG, RESOURCE_KEY_DW},
        automation_condition=COLD_START_CONDITION_SKIP_DEPS,
    )

    return lana_to_dw_protoasset


def ensure_target_table_exists_bq(
    context: dg.AssetExecutionContext,
    lana_core_pg: PostgresResource,
    dw: BigQueryDWResource,
    table_name: str,
) -> None:
    """
    Ensure the target BigQuery table exists.

    If the table doesn't exist (because DLT didn't create it due to empty source),
    create it with schema inferred from the Postgres source table.
    """
    from src.utils import (
        create_empty_table,
        get_postgres_table_schema,
        postgres_schema_to_bigquery_schema,
        table_exists,
    )

    raw_schema = dw.get_raw_schema()
    bq_client = dw.get_client()

    if table_exists(bq_client, raw_schema, table_name):
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
        dataset=raw_schema,
        table_name=table_name,
        schema=bq_schema,
    )

    context.log.info(
        f"Created empty table {table_name} in BigQuery with {len(bq_schema)} columns."
    )


def prepare_lana_el_pipeline(
    lana_core_pg: PostgresResource,
    dw: Union[BigQueryDWResource, PostgresDWResource],
    table_name: str,
):
    """Prepare a dlt pipeline for loading data from lana-core to the data warehouse."""
    dlt_postgres_resource = create_dlt_postgres_resource(
        connection_string=lana_core_pg.get_connection_string(), table_name=table_name
    )
    
    dlt_destination = create_dw_destination(dw.get_credentials())
    raw_schema = dw.get_raw_schema()

    pipeline = dlt.pipeline(
        pipeline_name=table_name,
        destination=dlt_destination,
        dataset_name=raw_schema,
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
