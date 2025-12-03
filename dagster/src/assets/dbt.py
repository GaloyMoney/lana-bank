import json
from typing import List, Set, Tuple

from dagster_dbt import DbtCliResource

import dagster as dg
from src.core import Protoasset
from src.resources import RESOURCE_KEY_LANA_DBT, dbt_manifest_path


def _load_dbt_manifest() -> dict:
    """Load and parse the dbt manifest.json file."""
    with open(dbt_manifest_path, "r") as f:
        return json.load(f)


def _get_dbt_asset_key(manifest: dict, model_unique_id: str) -> List[str]:
    """
    Generate Dagster asset key for a dbt model.
    Format: [project_name, ...model_path_parts, model_name]
    """
    model_node = manifest["nodes"][model_unique_id]
    project_name = manifest["metadata"]["project_name"]
    
    # Extract model path parts from fqn (fully qualified name)
    # fqn format: [project_name, ...path_parts, model_name]
    fqn = model_node.get("fqn", [])
    if len(fqn) > 1:
        # Skip project_name (first element), use the rest
        return fqn
    else:
        # Fallback: just use project_name and model_name
        return [project_name, model_node["name"]]


def _get_dbt_model_dependencies(
    manifest: dict, 
    model_unique_id: str, 
    el_asset_keys: Set[Tuple[str, ...]]
) -> List[List[str]]:
    """Extract Dagster asset key dependencies for a dbt model.
    
    Returns a list of asset key lists (not strings) since Dagster requires
    multi-part asset keys to be specified as lists, not dot-separated strings.
    
    Includes:
    - Other dbt models (from depends_on.nodes)
    - Lana EL assets (from depends_on.sources, mapped to ["lana", table_name])
      Only includes EL assets that are in the provided el_asset_keys set.
    
    Args:
        manifest: The dbt manifest dictionary
        model_unique_id: The unique ID of the dbt model
        el_asset_keys: Set of EL asset keys (as tuples) that exist in Dagster.
                      Format: {("lana", "table_name"), ...}
    """
    model_node = manifest["nodes"][model_unique_id]
    deps = []
    
    # Get upstream dependencies
    depends_on = model_node.get("depends_on", {})
    
    # Add dependencies on other dbt models
    nodes = depends_on.get("nodes", [])
    for dep_unique_id in nodes:
        dep_node = manifest["nodes"].get(dep_unique_id)
        if dep_node and dep_node["resource_type"] == "model":
            # Generate asset key for dependency - return as list, not string
            dep_asset_key = _get_dbt_asset_key(manifest, dep_unique_id)
            deps.append(dep_asset_key)
    
    # Add dependencies on Lana EL assets (from sources)
    sources = depends_on.get("sources", [])
    for source_unique_id in sources:
        # Look up the source in the manifest's sources dictionary
        # Source unique_id format: "source.<project_name>.<source_name>.<table_name>"
        source_node = manifest.get("sources", {}).get(source_unique_id)
        
        table_name = None
        source_name = None
        
        if source_node:
            # Extract source name and table name from the source node
            source_name = source_node.get("source_name", "")
            table_name = source_node.get("name", "")
        else:
            # Fallback: try to parse from unique_id if source not found in manifest
            # This handles edge cases where the source might not be in manifest["sources"]
            source_parts = source_unique_id.split(".")
            if len(source_parts) >= 4 and source_parts[0] == "source":
                # Format: source.<project_name>.<source_name>.<table_name>
                source_name = source_parts[2]  # "lana"
                table_name = source_parts[3]   # "core_deposit_events_rollup"
        
        # Only add dependency if it's from the "lana" source and the asset exists
        if source_name == "lana" and table_name:
            # Map to Lana EL asset key format: ["lana", table_name]
            lana_el_asset_key = ["lana", table_name]
            # Convert to tuple for set lookup
            lana_el_asset_key_tuple = tuple(lana_el_asset_key)
            
            # Only add if this EL asset key exists in the provided set
            if lana_el_asset_key_tuple in el_asset_keys:
                deps.append(lana_el_asset_key)
    
    return deps


def _create_dbt_model_callable(manifest: dict, model_unique_id: str):
    """Create a callable that runs a specific dbt model."""
    model_node = manifest["nodes"][model_unique_id]
    fqn = model_node.get("fqn", [])
    # Use fqn for more specific model selection (handles models with same name in different paths)
    # Format: project_name.path.to.model_name
    model_selector = ".".join(fqn)
    
    def run_dbt_model(
        context: dg.AssetExecutionContext, dbt: DbtCliResource
    ) -> None:
        """Run a specific dbt model.
        
        Note: This function is wrapped by trace_callable in the assetifier,
        so OpenTelemetry tracing is applied automatically.
        """
        context.log.info(f"Running dbt model: {model_unique_id}")
        
        # Run the specific model using dbt's select syntax with fqn
        # Stream events so Dagster can see progress in real-time
        # The span context from trace_callable should be maintained here
        # Don't pass context to avoid Dagster trying to extract metadata from asset
        # (since our assets aren't created with @dbt_assets decorator)
        # We'll manually log events to context instead
        stream = dbt.cli(
            ["run", "--select", model_selector],
            manifest=manifest
        ).stream()
        
        # Consume all events from the stream and log important ones
        for event in stream:
            # Log important events to context for visibility
            # (Since we're not passing context to dbt.cli, we manually log events)
            if hasattr(event, 'message') and event.message:
                context.log.info(f"dbt: {event.message}")
        
        context.log.info(f"Completed dbt model: {model_unique_id}")

    return run_dbt_model


def lana_dbt_protoassets(el_asset_keys: Set[Tuple[str, ...]]) -> List[Protoasset]:
    """
    Create Protoassets for each dbt model in the manifest.
    Each model will have dependencies on:
    - Other dbt models (from dbt's depends_on.nodes)
    - Lana EL assets (from dbt's depends_on.sources, mapped to ["lana", table_name])
      Only includes EL assets that are in the provided el_asset_keys set.
    
    Args:
        el_asset_keys: Set of EL asset keys (as tuples) that exist in Dagster.
                      Format: {("lana", "table_name"), ...}
                      These are the upstream EL assets that dbt models can depend on.
    """
    manifest = _load_dbt_manifest()
    protoassets = []
    
    for unique_id, node in manifest["nodes"].items():
        if node["resource_type"] != "model":
            continue
        
        asset_key = _get_dbt_asset_key(manifest, unique_id)
        deps = _get_dbt_model_dependencies(manifest, unique_id, el_asset_keys)
        callable = _create_dbt_model_callable(manifest, unique_id)
        
        protoasset = Protoasset(
            key=asset_key,
            callable=callable,
            tags={"asset_type": "dbt_model", "dbt_model": node["name"]},
            deps=deps,
            required_resource_keys={RESOURCE_KEY_LANA_DBT},
        )
        
        protoassets.append(protoasset)
    
    return protoassets
