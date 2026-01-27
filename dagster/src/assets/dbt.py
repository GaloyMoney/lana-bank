"""dbt assets using official dagster-dbt integration."""

import json
from typing import Any, List, Mapping, Optional

import dagster as dg
from dagster_dbt import (
    DagsterDbtTranslator,
    DagsterDbtTranslatorSettings,
    DbtCliResource,
    dbt_assets,
)

from src.otel import trace_dbt_batch
from src.resources import DBT_MANIFEST_PATH

TAG_KEY_ASSET_TYPE = "asset_type"
TAG_VALUE_DBT_MODEL = "dbt_model"
TAG_VALUE_DBT_SEED = "dbt_seed"


def _load_dbt_manifest() -> dict:
    """Load and parse the dbt manifest.json file."""
    with open(DBT_MANIFEST_PATH, "r") as f:
        manifest = json.load(f)
    return manifest


def _get_dbt_asset_key(manifest: dict, node_unique_id: str) -> List[str]:
    """
    Generate Dagster asset key path for a dbt node (model or seed).

    This matches the default dagster-dbt DagsterDbtTranslator.get_asset_key() behavior:
    - For models: [schema, name] if schema configured, else [name]
    - For seeds: [name]

    Used by file_report.py to find dbt model dependencies.
    """
    node = manifest["nodes"][node_unique_id]
    node_name = node["name"]

    # Match default dagster-dbt behavior: use configured schema if present
    config = node.get("config", {})
    configured_schema = config.get("schema")

    if configured_schema:
        return [configured_schema, node_name]
    return [node_name]


class LanaDbtTranslator(DagsterDbtTranslator):
    """
    Custom translator for mapping dbt assets to Dagster assets.

    Handles:
    - Source asset mapping: dbt sources -> EL asset keys ([source_name, table_name])
    - Custom tags for models and seeds
    - Automation conditions (eager for models, none for seeds)

    Dependencies are resolved automatically by dagster-dbt when get_asset_key()
    maps dbt sources to existing Dagster assets (our EL assets).
    """

    def __init__(self, resource_type: str):
        """
        Args:
            resource_type: Either "model" or "seed" to determine tag values and automation
        """
        super().__init__(
            settings=DagsterDbtTranslatorSettings(enable_asset_checks=False)
        )
        self._resource_type = resource_type

    def get_asset_key(self, dbt_resource_props: Mapping[str, Any]) -> dg.AssetKey:
        """Generate asset key for dbt nodes.

        For sources, maps to EL asset keys ([source_name, table_name]) so that
        dagster-dbt automatically creates dependencies on our EL assets.
        """
        resource_type = dbt_resource_props.get("resource_type")

        # For sources, map to EL asset keys (e.g., ["lana", "core_customer_events_rollup"])
        # This enables automatic dependency resolution with our EL assets
        if resource_type == "source":
            source_name = dbt_resource_props.get("source_name")
            table_name = dbt_resource_props.get("name")
            return dg.AssetKey([source_name, table_name])

        # For models and seeds, use default behavior (fqn-based)
        return super().get_asset_key(dbt_resource_props)

    def get_tags(self, dbt_resource_props: Mapping[str, Any]) -> Mapping[str, str]:
        """Apply custom tags to dbt assets."""
        resource_type = dbt_resource_props.get("resource_type")
        node_name = dbt_resource_props.get("name", "")

        if resource_type == "model":
            return {
                TAG_KEY_ASSET_TYPE: TAG_VALUE_DBT_MODEL,
                "dbt_model": node_name,
            }
        elif resource_type == "seed":
            return {
                TAG_KEY_ASSET_TYPE: TAG_VALUE_DBT_SEED,
                "dbt_seed": node_name,
            }

        return {}

    def get_automation_condition(
        self, dbt_resource_props: Mapping[str, Any]
    ) -> Optional[dg.AutomationCondition]:
        """Set automation condition based on resource type."""
        resource_type = dbt_resource_props.get("resource_type")

        # Only models get eager automation, seeds run on schedule
        if resource_type == "model":
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
    translator = LanaDbtTranslator(resource_type="model")

    @dbt_assets(
        manifest=DBT_MANIFEST_PATH,
        select="resource_type:model",
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
    translator = LanaDbtTranslator(resource_type="seed")

    @dbt_assets(
        manifest=DBT_MANIFEST_PATH,
        select="resource_type:seed",
        dagster_dbt_translator=translator,
    )
    def lana_dbt_seed_assets(context: dg.AssetExecutionContext, dbt: DbtCliResource):
        """Execute dbt seeds with OTEL tracing."""
        selected_keys = [key.to_user_string() for key in context.selected_asset_keys]

        with trace_dbt_batch(context, "dbt_seeds_build", selected_keys):
            yield from dbt.cli(["build"], context=context).stream()

    return lana_dbt_seed_assets
