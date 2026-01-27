import os
import time
import uuid
from contextlib import contextmanager
from typing import Callable, Dict, Optional, Tuple, Union

from opentelemetry import trace
from opentelemetry.context import Context
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import SimpleSpanProcessor
from opentelemetry.trace.propagation.tracecontext import TraceContextTextMapPropagator

# traceparent for current job's span stored in run tags
JOB_TRACEPARENT_TAG = "otel.job.traceparent"
JOB_SPAN_LOCK_TAG = "otel.job.span_lock"


def init_telemetry():
    """Initialize OpenTelemetry tracer"""
    endpoint = os.getenv("OTEL_EXPORTER_OTLP_ENDPOINT")

    resource = Resource.create(
        {"service.name": "dagster-lana-dw", "service.namespace": "lana"}
    )

    provider = TracerProvider(resource=resource)

    otlp_exporter = OTLPSpanExporter(endpoint=endpoint, insecure=True)
    # Since our runtimes are ephemeral, we send spans immediately and not
    # batched so we don't have to worry about flushing them.
    provider.add_span_processor(SimpleSpanProcessor(otlp_exporter))

    trace.set_tracer_provider(provider)


tracer = trace.get_tracer(__name__)
_trace_propagator = TraceContextTextMapPropagator()


def trace_callable(
    span_name: str,
    callable: Callable,
    span_attributes: Union[dict, None] = None,
    parent_context: Optional[Context] = None,
):
    """
    Wrapper that traces a callable with OpenTelemetry.

    Args:
        span_name: Name for the trace span
        callable: The function to wrap with tracing
        span_attributes: Optional dict of attributes to set on the span
        parent_context: Optional opentelemetry context to use as the parent span context

    Returns:
        A callable that executes the original callable within a trace span
    """

    def traced_wrapper(**kwargs):
        with tracer.start_as_current_span(span_name, context=parent_context) as span:
            if span_attributes:
                for key, value in span_attributes.items():
                    span.set_attribute(key, str(value))

            try:
                result = callable(**kwargs)
                span.set_status(trace.Status(trace.StatusCode.OK))
                return result
            except Exception as e:
                span.set_status(trace.Status(trace.StatusCode.ERROR, str(e)))
                raise e

    return traced_wrapper


def get_asset_span_context_and_attrs(
    context: Context, asset_key_str: str
) -> Tuple[Optional[Context], Dict[str, str]]:
    """
    For a given asset execution, compute:
      - parent_context: the job-level span context (if any)
      - attrs: common attributes for the asset span
    """
    attrs: Dict[str, str] = {
        "asset.name": asset_key_str,
        "run.id": context.run_id,
    }

    parent_ctx: Optional[Context] = None
    job_name = _get_job_name(context)

    if job_name and job_name != "__ASSET_JOB":
        parent_ctx = _ensure_job_parent_context(context, job_name)
        attrs["job.name"] = job_name

    return parent_ctx, attrs


def _get_job_name(context) -> Optional[str]:
    return (
        getattr(context, "job_name", None)
        or getattr(getattr(context, "dagster_run", None), "job_name", None)
        or getattr(getattr(context, "run", None), "job_name", None)
    )


def _try_acquire_span_creation_lock(context) -> Optional[str]:
    """
    Try to acquire the exclusive right to create the job span for this run.

    Returns the claim_id if we won the lock, None otherwise.

    This prevents race conditions when multiple processes try to create
    the job span simultaneously by using optimistic locking with a unique UUID.
    """
    claim_id = str(uuid.uuid4())

    try:
        context.instance.add_run_tags(context.run_id, {JOB_SPAN_LOCK_TAG: claim_id})
    except Exception:
        # Failed to write lock tag
        return None

    # Small delay to ensure tag write propagates through Dagster's storage
    time.sleep(0.05)

    # Check if we won the lock (first write wins)
    tags = dict(
        getattr(context.instance.get_run_by_id(context.run_id), "tags", {}) or {}
    )

    return claim_id if tags.get(JOB_SPAN_LOCK_TAG) == claim_id else None


def _wait_for_job_traceparent(context, timeout_seconds: float = 1.0) -> Optional[str]:
    """
    Wait for another process to create and persist the job traceparent.

    Returns the traceparent if found within timeout, None otherwise.
    """
    attempts = int(timeout_seconds / 0.05)

    for _ in range(attempts):
        time.sleep(0.05)
        tags = dict(
            getattr(context.instance.get_run_by_id(context.run_id), "tags", {}) or {}
        )
        if traceparent := tags.get(JOB_TRACEPARENT_TAG):
            return traceparent

    return None


def _ensure_job_parent_context(context, job_name: str) -> Optional[Context]:
    """
    Ensure there is exactly one job-level span per Dagster run.
    We persist the span context via run tags so all processes can reuse it.

    If a traceparent is passed via run tags (e.g., from a sensor), we reuse it
    to continue the same trace across multiple jobs.
    """
    # Check if job span already exists (or was passed from another job)
    tags = dict(
        getattr(context.instance.get_run_by_id(context.run_id), "tags", {}) or {}
    )
    if existing := tags.get(JOB_TRACEPARENT_TAG):
        return _context_from_traceparent(existing)

    # Try to acquire lock to create a new job span
    if claim_id := _try_acquire_span_creation_lock(context):
        # We won the lock - create the job span
        with tracer.start_as_current_span(
            job_name,
            attributes={"job.name": job_name, "run.id": context.run_id},
        ):
            traceparent = _current_span_to_traceparent()

        if traceparent:
            context.instance.add_run_tags(
                context.run_id, {JOB_TRACEPARENT_TAG: traceparent}
            )
            return _context_from_traceparent(traceparent)

    # We lost the lock - wait for winner to write traceparent
    if traceparent := _wait_for_job_traceparent(context):
        return _context_from_traceparent(traceparent)

    # Timeout - winner may have failed
    return None


def _context_from_traceparent(traceparent: str) -> Context:
    return _trace_propagator.extract(carrier={"traceparent": traceparent})


def _current_span_to_traceparent() -> Optional[str]:
    carrier: Dict[str, str] = {}
    _trace_propagator.inject(carrier)
    return carrier.get("traceparent")


@contextmanager
def trace_dbt_batch(context, batch_name: str, selected_keys: list):
    """
    Context manager for tracing a dbt batch execution with OpenTelemetry.

    Creates a span for the batch execution, integrating with the job-level
    parent span if available.

    Args:
        context: Dagster AssetExecutionContext
        batch_name: Name for the span (e.g., "dbt_models_build", "dbt_seeds_build")
        selected_keys: List of asset key strings being materialized

    Yields:
        The active span for additional attribute setting if needed
    """
    attrs: Dict[str, str] = {
        "dbt.batch_name": batch_name,
        "dbt.model_count": str(len(selected_keys)),
        "dbt.models": ", ".join(selected_keys[:10]),  # Limit to first 10 for readability
        "run.id": context.run_id,
    }

    parent_ctx: Optional[Context] = None
    job_name = _get_job_name(context)

    if job_name and job_name != "__ASSET_JOB":
        parent_ctx = _ensure_job_parent_context(context, job_name)
        attrs["job.name"] = job_name

    with tracer.start_as_current_span(batch_name, context=parent_ctx) as span:
        for key, value in attrs.items():
            span.set_attribute(key, value)

        try:
            yield span
            span.set_status(trace.Status(trace.StatusCode.OK))
        except Exception as e:
            span.set_status(trace.Status(trace.StatusCode.ERROR, str(e)))
            raise
