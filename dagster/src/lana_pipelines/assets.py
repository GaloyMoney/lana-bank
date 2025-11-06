from typing import Any, Optional, Mapping

import dlt
import dagster as dg
from dagster_dbt import DbtCliResource, dbt_assets, DagsterDbtTranslator
from generate_es_reports.service import run_report_batch

from lana_pipelines.resources import (
    dbt_manifest_path,
    PostgresResource,
    BigQueryResource,
)
from lana_pipelines.dlt.pipelines import prepare_lana_el_pipeline
from lana_pipelines import constants


def build_all_lana_source_assets(table_names):
    lana_source_assets = []
    for table_name in table_names:
        lana_source_assets.append(
            dg.AssetSpec(
                key=f"el_source_asset__lana__{table_name}",
                tags={"asset_type": "el_source__asset", "system": "lana"},
            )
        )
    return lana_source_assets


def build_all_lana_to_dw_el_assets(table_names):

    lana_el_assets = []
    for table_name in table_names:
        lana_el_assets.append(
            build_lana_to_dw_el_asset(
                table_name=table_name,
            )
        )

    return lana_el_assets


def build_lana_to_dw_el_asset(table_name):

    @dg.asset(
        key_prefix=["lana"],
        name=table_name,
        deps=[f"el_source_asset__lana__{table_name}"],
        tags={"asset_type": "el_target__asset", "system": "lana"},
    )
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

    return lana_to_dw_el_asset


def build_dbt_assets():

    class CustomDagsterDbtTranslator(DagsterDbtTranslator):
        def get_automation_condition(
            self, dbt_resource_props: Mapping[str, Any]
        ) -> Optional[dg.AutomationCondition]:
            return dg.AutomationCondition.eager()

    @dbt_assets(
        manifest=dbt_manifest_path, dagster_dbt_translator=CustomDagsterDbtTranslator()
    )
    def dbt_models(context: dg.AssetExecutionContext, dbt: DbtCliResource):
        yield from dbt.cli(["build"], context=context).stream()

    return dbt_models


def build_generate_es_report_asset():

    @dg.asset(deps=["report_uif_07_diario_otros_medios_electronicos"])
    def generate_es_report_asset():
        run_report_batch()

    return generate_es_report_asset
