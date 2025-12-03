import json
from typing import List, Optional, Set, Tuple

from dagster_dbt import DbtCliResource

import dagster as dg
from src.core import Protoasset
from src.resources import DBT_MANIFEST_PATH, RESOURCE_KEY_LANA_DBT


def _load_dbt_manifest() -> dict:
    """Load and parse the dbt manifest.json file."""
    with open(DBT_MANIFEST_PATH, "r") as f:
        manifest = json.load(f)
    return manifest


def _get_dbt_asset_key(manifest: dict, model_unique_id: str) -> List[str]:
    """
    Generate Dagster asset key for a dbt model.
    Format: [project_name, ...model_path_parts, model_name]
    """
    model_fully_qualified_name: list[str] = manifest["nodes"][model_unique_id].get(
        "fqn", []
    )
    model_name = manifest["nodes"][model_unique_id]["name"]
    project_name = manifest["metadata"]["project_name"]

    has_project_name = len(model_fully_qualified_name) > 1
    if has_project_name:
        return model_fully_qualified_name
    if not has_project_name:
        return [project_name, model_name]


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

    # Add dependencies on other dbt models
    for dep_unique_id in depends_on.get("nodes", []):
        dep_node = manifest["nodes"].get(dep_unique_id)
        if dep_node and dep_node["resource_type"] == "model":
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
        if isinstance(protoasset.key, list):
            source_asset_keys.add(tuple(protoasset.key))
        else:
            source_asset_keys.add((protoasset.key,))

    manifest = _load_dbt_manifest()
    dbt_protoassets = []

    for unique_id, node in manifest["nodes"].items():
        if node["resource_type"] != "model":
            continue

        asset_key = _get_dbt_asset_key(manifest, unique_id)
        deps = _get_dbt_model_dependencies(manifest, unique_id, source_asset_keys)
        callable = _create_dbt_model_callable(manifest, unique_id)

        protoasset = Protoasset(
            key=asset_key,
            callable=callable,
            tags={"asset_type": "dbt_model", "dbt_model": node["name"]},
            deps=deps,
            required_resource_keys={RESOURCE_KEY_LANA_DBT},
        )

        dbt_protoassets.append(protoasset)

    return dbt_protoassets
