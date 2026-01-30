import json
from enum import StrEnum
from typing import Any, List, Mapping, Optional

from dagster_dbt import (
    DagsterDbtTranslator,
    DagsterDbtTranslatorSettings,
    DbtCliResource,
    dbt_assets,
)

import dagster as dg
from src.otel import trace_dbt_batch
from src.resources import DBT_MANIFEST_PATH


class DbtResourceType(StrEnum):
    MODEL = "model"
    SEED = "seed"
    SOURCE = "source"


class DbtPropKey(StrEnum):
    RESOURCE_TYPE = "resource_type"
    SOURCE_NAME = "source_name"
    NAME = "name"
    FQN = "fqn"


DBT_SEEDS_FOLDER = "seeds"

TAG_KEY_ASSET_TYPE = "asset_type"
TAG_VALUE_DBT_MODEL = "dbt_model"
TAG_VALUE_DBT_SEED = "dbt_seed"

DBT_SELECT_MODELS = f"{DbtPropKey.RESOURCE_TYPE}:{DbtResourceType.MODEL}"
DBT_SELECT_SEEDS = f"{DbtPropKey.RESOURCE_TYPE}:{DbtResourceType.SEED}"


def _load_dbt_manifest() -> dict:
    """Load and parse the dbt manifest.json file."""
    with open(DBT_MANIFEST_PATH, "r") as f:
        manifest = json.load(f)
    return manifest


def _get_dbt_asset_key(manifest: dict, node_unique_id: str) -> List[str]:
    """
    Generate Dagster asset key path for a dbt node using its fqn.

    Format:
    - Models: fqn as-is, e.g. ["dbt_lana_dw", "staging", "rollups", "model_name"]
    - Seeds:  fqn with seeds folder, e.g. ["dbt_lana_dw", "seeds", "seed_name"]

    Used by file_report.py and LanaDbtTranslator to produce consistent asset keys.
    """
    node = manifest["nodes"][node_unique_id]
    fqn: list[str] = node.get(DbtPropKey.FQN, [])
    node_name = node[DbtPropKey.NAME]
    project_name = manifest["metadata"]["project_name"]
    resource_type = node.get(DbtPropKey.RESOURCE_TYPE, DbtResourceType.MODEL)

    if resource_type == DbtResourceType.SEED:
        if len(fqn) >= 2 and fqn[1] != DBT_SEEDS_FOLDER:
            return [fqn[0], DBT_SEEDS_FOLDER] + fqn[1:]
        elif len(fqn) == 1:
            return [project_name, DBT_SEEDS_FOLDER, node_name]
        return fqn

    if len(fqn) > 1:
        return fqn
    return [project_name, node_name]


class LanaDbtTranslator(DagsterDbtTranslator):
    """
    Custom translator for mapping dbt assets to Dagster assets.
    """

    def __init__(self, resource_type: DbtResourceType):
        """
        Args:
            resource_type: DbtResourceType.MODEL or DbtResourceType.SEED
        """
        super().__init__(
            settings=DagsterDbtTranslatorSettings(enable_asset_checks=False)
        )
        self._resource_type = resource_type

    def get_asset_key(self, dbt_resource_props: Mapping[str, Any]) -> dg.AssetKey:
        """Generate asset key for dbt nodes using fqn-based paths.

        - Sources: EL asset keys ([source_name, table_name])
        - Models: fqn as-is, e.g. ["dbt_lana_dw", "staging", "rollups", "model_name"]
        - Seeds:  fqn with seeds folder, e.g. ["dbt_lana_dw", "seeds", "seed_name"]
        """
        resource_type = dbt_resource_props.get(DbtPropKey.RESOURCE_TYPE)

        # For sources, map to EL asset keys (e.g., ["lana", "core_customer_events_rollup"])
        # This enables automatic dependency resolution with our EL assets
        if resource_type == DbtResourceType.SOURCE:
            source_name = dbt_resource_props.get(DbtPropKey.SOURCE_NAME)
            table_name = dbt_resource_props.get(DbtPropKey.NAME)
            return dg.AssetKey([source_name, table_name])

        fqn = list(dbt_resource_props.get(DbtPropKey.FQN, []))
        node_name = dbt_resource_props.get(DbtPropKey.NAME)

        if resource_type == DbtResourceType.SEED:
            if len(fqn) >= 2 and fqn[1] != DBT_SEEDS_FOLDER:
                return dg.AssetKey([fqn[0], DBT_SEEDS_FOLDER] + fqn[1:])
            elif len(fqn) == 1:
                return dg.AssetKey([fqn[0], DBT_SEEDS_FOLDER, node_name])
            return dg.AssetKey(fqn)

        if resource_type == DbtResourceType.MODEL:
            if len(fqn) > 1:
                return dg.AssetKey(fqn)
            return dg.AssetKey([fqn[0], node_name] if fqn else [node_name])

        raise ValueError(f"Can't handle resource_type: {resource_type}")

    def get_tags(self, dbt_resource_props: Mapping[str, Any]) -> Mapping[str, str]:
        """Apply custom tags to dbt assets."""
        resource_type = dbt_resource_props.get(DbtPropKey.RESOURCE_TYPE)
        node_name = dbt_resource_props.get(DbtPropKey.NAME, "")

        if resource_type == DbtResourceType.MODEL:
            return {
                TAG_KEY_ASSET_TYPE: TAG_VALUE_DBT_MODEL,
                TAG_VALUE_DBT_MODEL: node_name,
            }
        elif resource_type == DbtResourceType.SEED:
            return {
                TAG_KEY_ASSET_TYPE: TAG_VALUE_DBT_SEED,
                TAG_VALUE_DBT_SEED: node_name,
            }

        return {}

    def get_automation_condition(
        self, dbt_resource_props: Mapping[str, Any]
    ) -> Optional[dg.AutomationCondition]:
        """Set automation condition based on resource type."""
        resource_type = dbt_resource_props.get(DbtPropKey.RESOURCE_TYPE)

        # Only models get eager automation, seeds run on schedule
        if resource_type == DbtResourceType.MODEL:
            return dg.AutomationCondition.eager()

        return None


def create_dbt_model_assets():
    """
    Create dbt model assets using the official @dbt_assets decorator.

    Dependencies on EL assets (like lana/*, bitfinex/*, sumsub/*) are resolved
    automatically by dagster-dbt through the get_asset_key() mapping for sources.

    Returns:
        A dbt_assets-decorated function that creates all dbt model assets
    """
    translator = LanaDbtTranslator(resource_type=DbtResourceType.MODEL)

    @dbt_assets(
        manifest=DBT_MANIFEST_PATH,
        select=DBT_SELECT_MODELS,
        dagster_dbt_translator=translator,
    )
    def lana_dbt_model_assets(context: dg.AssetExecutionContext, dbt: DbtCliResource):
        """Execute dbt models with OTEL tracing."""
        selected_keys = [key.to_user_string() for key in context.selected_asset_keys]

        with trace_dbt_batch(context, "dbt_models_build", selected_keys):
            yield from dbt.cli(["run"], context=context).stream()

    return lana_dbt_model_assets


def create_dbt_seed_assets():
    """
    Create dbt seed assets using the official @dbt_assets decorator.

    Seeds run on a schedule (no automation condition).

    Returns:
        A dbt_assets-decorated function that creates all dbt seed assets
    """
    translator = LanaDbtTranslator(resource_type=DbtResourceType.SEED)

    @dbt_assets(
        manifest=DBT_MANIFEST_PATH,
        select=DBT_SELECT_SEEDS,
        dagster_dbt_translator=translator,
    )
    def lana_dbt_seed_assets(context: dg.AssetExecutionContext, dbt: DbtCliResource):
        """Execute dbt seeds with OTEL tracing."""
        selected_keys = [key.to_user_string() for key in context.selected_asset_keys]

        with trace_dbt_batch(context, "dbt_seeds_build", selected_keys):
            yield from dbt.cli(["seed"], context=context).stream()

    return lana_dbt_seed_assets
