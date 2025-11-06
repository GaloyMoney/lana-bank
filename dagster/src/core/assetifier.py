"""Asset wrapping logic for Lana Dagster project."""

from typing import Callable

import dagster as dg

from src.otel import trace_callable


def lana_assetifier(asset_key: str, callable: Callable):
    """
    Gets a callable, applies centralized wrapping specific to our project,
    returns a dg.asset out of it.

    Args:
        asset_key: The dagster asset key
        callable (Callable): the callable to materialize the asset.

    Returns:
        A Dagster asset with all Lana-specific wrapping applied
    """
    @dg.asset(key=asset_key)
    def wrapped_callable(context: dg.AssetExecutionContext) -> None:
        asset_key_str: str = context.asset_key.to_user_string()

        span_name = f"asset_{asset_key_str}_run_{context.run_id}"
        span_attributes = {
            "asset.name": asset_key_str,
            "run.id": context.run_id
        }

        traced_callable = trace_callable(
            span_name=span_name,
            callable=callable,
            span_attributes=span_attributes
        )

        traced_callable(context=context)

    return wrapped_callable

