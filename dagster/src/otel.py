import os
from typing import Callable, Dict, Optional, Tuple, Union

from opentelemetry import trace
from opentelemetry.context import Context
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import SimpleSpanProcessor
from opentelemetry.trace.propagation.tracecontext import TraceContextTextMapPropagator


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
JOB_TRACEPARENT_TAG = "otel.job.traceparent"
PARENT_JOB_TRACEPARENT_TAG = "otel.parent_job.traceparent"


def _context_from_traceparent(traceparent: str) -> Context:
    return _trace_propagator.extract(carrier={"traceparent": traceparent})


def _current_span_to_traceparent() -> Optional[str]:
    carrier: Dict[str, str] = {}
    _trace_propagator.inject(carrier)
    return carrier.get("traceparent")


def _get_job_name(context) -> Optional[str]:
    job_name = getattr(context, "job_name", None)
    if not job_name and hasattr(context, "dagster_run"):
        job_name = getattr(context.dagster_run, "job_name", None)
    if not job_name and hasattr(context, "run"):
        job_name = getattr(context.run, "job_name", None)
    return job_name


def _ensure_job_parent_context(context, job_name: str) -> Optional[Context]:
    """
    Ensure there is exactly one job-level span per Dagster run.

    We persist the span context via run tags so all processes can reuse it.
    If a parent job traceparent is provided, use it as the parent context.
    """
    run = context.instance.get_run_by_id(context.run_id)
    tags = dict(getattr(run, "tags", {}) or {})

    existing = tags.get(JOB_TRACEPARENT_TAG)
    if existing:
        return _context_from_traceparent(existing)

    # Check if there's a parent job traceparent to use as parent context
    parent_traceparent = tags.get(PARENT_JOB_TRACEPARENT_TAG)
    parent_context = None
    if parent_traceparent:
        parent_context = _context_from_traceparent(parent_traceparent)

    # No job span recorded yet for this run – create one now.
    span_kwargs = {}
    if parent_context is not None:
        span_kwargs["context"] = parent_context

    with tracer.start_as_current_span(
        job_name,
        attributes={"job.name": job_name, "run.id": context.run_id},
        **span_kwargs,
    ):
        traceparent = _current_span_to_traceparent()

    if traceparent:
        context.instance.add_run_tags(
            context.run_id, {JOB_TRACEPARENT_TAG: traceparent}
        )
        return _context_from_traceparent(traceparent)

    return None


def get_asset_span_context_and_attrs(
    context, asset_key_str: str
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
        parent_context: Optional OTEL Context to use as the parent span context

    Returns:
        A callable that executes the original callable within a trace span
    """

    def traced_wrapper(**kwargs):
        span_kwargs = {}
        if parent_context is not None:
            span_kwargs["context"] = parent_context

        with tracer.start_as_current_span(span_name, **span_kwargs) as span:
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
