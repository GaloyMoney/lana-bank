"""Asset wrapping logic for Lana Dagster project."""

from typing import TYPE_CHECKING, Union

import dagster as dg
from src.otel import get_asset_span_context_and_attrs, trace_callable

if TYPE_CHECKING:
    from .protoasset import Protoasset


def lana_assetifier(protoasset: "Protoasset") -> Union[dg.asset, dg.AssetSpec]:
    """
    Gets a protoasset, applies centralized wrapping specific to our project,
    returns a dg.asset out of it.

    Args:
        protoasset (Protoasset): a protoasset of the project.

    Returns:
        A Dagster asset with all Lana-specific wrapping applied
    """

    if protoasset.is_external:
        asset = dg.AssetSpec(key=protoasset.key, tags=protoasset.tags)
        return asset

    @dg.asset(
        key=protoasset.key,
        tags=protoasset.tags,
        deps=protoasset.deps,
        required_resource_keys=protoasset.required_resource_keys,
        automation_condition=protoasset.automation_condition,
    )
    def wrapped_callable(context: dg.AssetExecutionContext):
        asset_key_str: str = context.asset_key.to_user_string()

        parent_context, span_attributes = get_asset_span_context_and_attrs(
            context, asset_key_str
        )

        span_name = f"asset_{asset_key_str}_run"

        traced_callable = trace_callable(
            span_name=span_name,
            callable=protoasset.callable,
            span_attributes=span_attributes,
            parent_context=parent_context,
        )

        # Extract resources from context.resources and pass them to the callable
        callable_kwargs = {"context": context}
        if protoasset.required_resource_keys:
            for resource_key in protoasset.required_resource_keys:
                callable_kwargs[resource_key] = getattr(context.resources, resource_key)

        return traced_callable(**callable_kwargs)

    return wrapped_callable
