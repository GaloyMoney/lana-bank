import os
from typing import Callable, Union

from opentelemetry import trace
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import SimpleSpanProcessor


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


def trace_callable(
    span_name: str, callable: Callable, span_attributes: Union[dict, None] = None
):
    """
    Wrapper that traces a callable with OpenTelemetry.

    Args:
        span_name: Name for the trace span
        callable: The function to wrap with tracing
        span_attributes: Optional dict of attributes to set on the span

    Returns:
        A callable that executes the original callable within a trace span
    """

    def traced_wrapper(**kwargs):
        with tracer.start_as_current_span(span_name) as span:
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
