"""Asset wrapping logic for Lana Dagster project."""

from typing import TYPE_CHECKING

import dagster as dg
from src.otel import trace_callable

if TYPE_CHECKING:
    from .assetifier import Protoasset


def lana_assetifier(protoasset: "Protoasset") -> dg.asset:
    """
    Gets a protoasset, applies centralized wrapping specific to our project,
    returns a dg.asset out of it.

    Args:
        protoasset (Protoasset): a protoasset of the project.

    Returns:
        A Dagster asset with all Lana-specific wrapping applied
    """

    @dg.asset(key=protoasset.key)
    def wrapped_callable(context: dg.AssetExecutionContext) -> None:
        asset_key_str: str = context.asset_key.to_user_string()

        span_name = f"asset_{asset_key_str}_run"
        span_attributes = {"asset.name": asset_key_str, "run.id": context.run_id}

        traced_callable = trace_callable(
            span_name=span_name,
            callable=protoasset.callable,
            span_attributes=span_attributes,
        )

        traced_callable(context=context)

    return wrapped_callable
