import os
from typing import Callable, Dict, Optional, Union

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


def context_from_traceparent(traceparent: str) -> Context:
    """Build an OTEL Context from a traceparent string."""
    return _trace_propagator.extract(carrier={"traceparent": traceparent})


def current_span_to_traceparent() -> Optional[str]:
    """
    Serialize the *current* span context into a W3C traceparent string.
    Returns None if nothing could be injected.
    """
    carrier: Dict[str, str] = {}
    _trace_propagator.inject(carrier)
    return carrier.get("traceparent")


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
