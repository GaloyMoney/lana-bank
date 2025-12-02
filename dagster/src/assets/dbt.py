from typing import Any, Mapping, Optional

from dagster_dbt import DagsterDbtTranslator, DbtCliResource, dbt_assets

import dagster as dg
from src.resources import dbt_manifest_path


def lana_dbt_assets():
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
