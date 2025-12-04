"""Asset wrapping logic for Lana Dagster project."""

from typing import TYPE_CHECKING, Dict, Optional, Union

from opentelemetry.context import Context
from opentelemetry.trace.propagation.tracecontext import TraceContextTextMapPropagator

import dagster as dg
from src.otel import (
    context_from_traceparent,
    current_span_to_traceparent,
    trace_callable,
    tracer,
)

if TYPE_CHECKING:
    from .protoasset import Protoasset


# Run tag key used to persist the job-level trace context
JOB_TRACEPARENT_TAG = "otel.job.traceparent"
_trace_propagator = TraceContextTextMapPropagator()


def _get_job_name_from_context(context: dg.AssetExecutionContext) -> Optional[str]:
    # Prefer a direct property if Dagster exposes one; otherwise fall back to dagster_run.
    job_name = getattr(context, "job_name", None)
    if not job_name and hasattr(context, "dagster_run"):
        job_name = getattr(context.dagster_run, "job_name", None)
    if not job_name and hasattr(context, "run"):
        job_name = getattr(context.run, "job_name", None)
    return job_name


def _get_run_tags_from_instance(context: dg.AssetExecutionContext) -> Dict[str, str]:
    """
    Always go through DagsterInstance so all processes see the same, updated tags.
    """
    run = context.instance.get_run_by_id(context.run_id)
    if run and run.tags:
        # tags is Mapping[str, str]
        return dict(run.tags)
    return {}


def _ensure_job_parent_context(
    context: dg.AssetExecutionContext, job_name: str
) -> Optional[Context]:
    """
    Ensure there's a job-level span for this Dagster run and return a parent Context.
    - If a traceparent tag already exists for this run, rebuild Context from it.
    - Otherwise, create a job span (named after the job), serialize its context
      to traceparent, store it as a run tag, and return a Context built from it.
    """
    run_tags = _get_run_tags_from_instance(context)
    existing_traceparent = run_tags.get(JOB_TRACEPARENT_TAG)

    if existing_traceparent:
        # Rebuild a Context from the stored traceparent
        return context_from_traceparent(existing_traceparent)

    # No job span recorded yet for this run – create one now.
    # Parent span name = job name, as requested.
    span_name = job_name

    with tracer.start_as_current_span(
        span_name,
        attributes={
            "job.name": job_name,
            "run.id": context.run_id,
        },
    ):
        # Serialize the current span context into a W3C traceparent
        traceparent = current_span_to_traceparent()

    if traceparent:
        # Persist to run tags so other processes / assets in this run can use it
        context.instance.add_run_tags(
            context.run_id, {JOB_TRACEPARENT_TAG: traceparent}
        )
        return context_from_traceparent(traceparent)

    return None


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
    )
    def wrapped_callable(context: dg.AssetExecutionContext):
        asset_key_str: str = context.asset_key.to_user_string()

        # Figure out if this run is coming from a named Dagster job
        job_name = _get_job_name_from_context(context)

        parent_context = None
        if job_name and job_name != "__ASSET_JOB":
            # Only create a parent span if this is actually a job-triggered run.
            parent_context = _ensure_job_parent_context(context, job_name)

        span_name = f"asset_{asset_key_str}_run"
        span_attributes = {
            "asset.name": asset_key_str,
            "run.id": context.run_id,
        }
        if job_name:
            span_attributes["job.name"] = job_name

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
