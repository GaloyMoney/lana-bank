import json
from typing import List, Optional, Set, Tuple

from dagster_dbt import DbtCliResource

import dagster as dg
from src.core import Protoasset
from src.resources import DBT_MANIFEST_PATH, RESOURCE_KEY_LANA_DBT

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
    Generate Dagster asset key for a dbt node (model or seed).
    Format: [project_name, ...path_parts, node_name]

    For seeds, inserts "seeds" folder into path for consistent grouping.
    """
    node = manifest["nodes"][node_unique_id]
    fqn: list[str] = node.get("fqn", [])
    node_name = node["name"]
    project_name = manifest["metadata"]["project_name"]
    resource_type = node.get("resource_type", "model")

    # Seeds: ensure they're under a "seeds" folder
    # dbt fqn for seeds is typically [project_name, seed_name]
    # We want [project_name, "seeds", seed_name]
    if resource_type == "seed":
        if len(fqn) >= 2 and fqn[1] != "seeds":
            return [fqn[0], "seeds"] + fqn[1:]
        elif len(fqn) == 1:
            return [project_name, "seeds", node_name]
        return fqn

    # Models: use fqn as-is (already has folder structure)
    if len(fqn) > 1:
        return fqn
    return [project_name, node_name]


def _get_source_dependencies(manifest: dict, model_unique_id: str) -> List[str]:
    """Extract source unique_ids that a model depends on.

    Sources can appear in multiple places in the dbt manifest:
    - depends_on.sources (standard location)
    - depends_on.nodes (dbt sometimes puts sources here)
    - parent_map (fallback location)
    """
    model_node_dependencies = manifest["nodes"][model_unique_id].get("depends_on", {})

    sources = list(model_node_dependencies.get("sources", []))

    for dep_id in model_node_dependencies.get("nodes", []):
        if dep_id.startswith("source."):
            sources.append(dep_id)

    if not sources:
        parent_map = manifest.get("parent_map", {})
        if model_unique_id in parent_map:
            sources = [
                p for p in parent_map[model_unique_id] if p.startswith("source.")
            ]

    return sources


def _extract_source_info(
    manifest: dict, source_unique_id: str
) -> Optional[Tuple[str, str]]:
    """Extract source_name and table_name from a source unique_id.

    Returns (source_name, table_name) if successful, None otherwise.
    """
    # Try to get from manifest sources dictionary
    source_node = manifest.get("sources", {}).get(source_unique_id)
    if source_node:
        return source_node.get("source_name"), source_node.get("name")

    # Fallback: parse from unique_id format: "source.<project>.<source_name>.<table_name>"
    parts = source_unique_id.split(".")
    if len(parts) >= 4 and parts[0] == "source":
        source_name = parts[2]
        table_name = parts[3]
        return source_name, table_name

    return None


def _get_dbt_model_dependencies(
    manifest: dict, model_unique_id: str, source_asset_keys: Set[Tuple[str, ...]]
) -> List[dg.AssetKey]:
    """Extract Dagster asset key dependencies for a dbt model.

    Returns a list of AssetKey objects for dependencies.

    Includes:
    - Other dbt models (from depends_on.nodes)
    - dbt seeds (from depends_on.nodes)
    - Lana source assets (from sources, mapped to ["lana", table_name])
      Only includes source assets that are in the provided source_asset_keys set.

    Args:
        manifest: The dbt manifest dictionary
        model_unique_id: The unique ID of the dbt model
        source_asset_keys: Set of source asset keys (as tuples) that exist in Dagster.
                      Format: {("lana", "table_name"), ...}
    """
    model_node = manifest["nodes"][model_unique_id]
    depends_on = model_node.get("depends_on", {})
    deps = []

    # Add dependencies on other dbt models and seeds
    for dep_unique_id in depends_on.get("nodes", []):
        dep_node = manifest["nodes"].get(dep_unique_id)
        if dep_node and dep_node["resource_type"] in ("model", "seed"):
            asset_key_list = _get_dbt_asset_key(manifest, dep_unique_id)
            deps.append(dg.AssetKey(asset_key_list))

    # Add dependencies on source assets
    for source_unique_id in _get_source_dependencies(manifest, model_unique_id):
        source_info = _extract_source_info(manifest, source_unique_id)
        if not source_info:
            continue

        source_name, table_name = source_info
        if not source_name or not table_name:
            continue

        source_asset_key_tuple = (source_name, table_name)
        if source_asset_key_tuple in source_asset_keys:
            deps.append(dg.AssetKey([source_name, table_name]))

    return deps


def _create_dbt_model_callable(manifest: dict, model_unique_id: str):
    """Create a callable that runs a specific dbt model."""
    fqn = manifest["nodes"][model_unique_id].get("fqn", [])
    # Use fqn for more specific model selection (handles models with same name in different paths)
    # Format: project_name.path.to.model_name
    model_selector = ".".join(fqn)

    def run_dbt_model(context: dg.AssetExecutionContext, dbt: DbtCliResource) -> None:
        """Run a specific dbt model."""
        context.log.info(f"Running dbt model: {model_unique_id}")

        stream = dbt.cli(
            ["run", "--select", model_selector], manifest=manifest
        ).stream()

        for event in stream:
            if hasattr(event, "message") and event.message:
                context.log.info(f"dbt: {event.message}")

        context.log.info(f"Completed dbt model: {model_unique_id}")

    return run_dbt_model


def _create_dbt_seed_callable(manifest: dict, seed_unique_id: str):
    """Create a callable that runs a specific dbt seed."""
    seed_name = manifest["nodes"][seed_unique_id]["name"]

    def run_dbt_seed(context: dg.AssetExecutionContext, dbt: DbtCliResource) -> None:
        """Run a specific dbt seed."""
        context.log.info(f"Running dbt seed: {seed_unique_id}")

        stream = dbt.cli(
            ["seed", "--select", seed_name], manifest=manifest
        ).stream()

        for event in stream:
            if hasattr(event, "message") and event.message:
                context.log.info(f"dbt: {event.message}")

        context.log.info(f"Completed dbt seed: {seed_unique_id}")

    return run_dbt_seed


def lana_dbt_protoassets(source_protoassets: List[Protoasset]) -> List[Protoasset]:
    """
    Create Protoassets for each dbt model in the manifest.
    Each model will have dependencies on:
    - Other dbt models (from dbt's depends_on.nodes)
    - EL assets (from dbt's depends_on.sources, mapped to [source_name, table_name])
      Only includes source assets that are in the provided source_protoassets list.

    Args:
        source_protoassets: List of source protoassets that exist in Dagster.
                       These are the upstream source assets that dbt models can depend on.
    """
    source_asset_keys = set()
    for protoasset in source_protoassets:
        # protoasset.key is an AssetKey, convert path (list) to tuple for set
        source_asset_keys.add(tuple(protoasset.key.path))

    manifest = _load_dbt_manifest()
    dbt_protoassets = []

    for unique_id, node in manifest["nodes"].items():
        if node["resource_type"] != "model":
            continue

        asset_key = _get_dbt_asset_key(manifest, unique_id)
        deps = _get_dbt_model_dependencies(manifest, unique_id, source_asset_keys)
        callable = _create_dbt_model_callable(manifest, unique_id)

        protoasset = Protoasset(
            key=dg.AssetKey(asset_key),
            callable=callable,
            tags={TAG_KEY_ASSET_TYPE: TAG_VALUE_DBT_MODEL, "dbt_model": node["name"]},
            deps=deps,
            required_resource_keys={RESOURCE_KEY_LANA_DBT},
            automation_condition=dg.AutomationCondition.eager(),
        )

        dbt_protoassets.append(protoasset)

    return dbt_protoassets


def lana_dbt_seed_protoassets() -> List[Protoasset]:
    """
    Create Protoassets for each dbt seed in the manifest.

    Seeds have no upstream dependencies and no automation_condition
    (they run on a schedule, not reactively).
    """
    manifest = _load_dbt_manifest()
    seed_protoassets = []

    for unique_id, node in manifest["nodes"].items():
        if node["resource_type"] != "seed":
            continue

        asset_key = _get_dbt_asset_key(manifest, unique_id)
        callable = _create_dbt_seed_callable(manifest, unique_id)

        protoasset = Protoasset(
            key=dg.AssetKey(asset_key),
            callable=callable,
            tags={TAG_KEY_ASSET_TYPE: TAG_VALUE_DBT_SEED, "dbt_seed": node["name"]},
            deps=[],
            required_resource_keys={RESOURCE_KEY_LANA_DBT},
            automation_condition=None,
        )

        seed_protoassets.append(protoasset)

    return seed_protoassets
