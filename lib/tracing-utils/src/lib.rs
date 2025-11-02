#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

use opentelemetry::{KeyValue, global, trace::TracerProvider};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    Resource,
    propagation::TraceContextPropagator,
    trace::{Sampler, SdkTracerProvider},
};
use opentelemetry_semantic_conventions::resource::SERVICE_NAMESPACE;
use serde::{Deserialize, Serialize};
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub use tracing::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TracingConfig {
    service_name: String,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            service_name: "lana-dev".to_string(),
        }
    }
}

pub fn init_tracer(config: TracingConfig) -> anyhow::Result<()> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()?;

    let provider = SdkTracerProvider::builder()
        .with_resource(telemetry_resource(&config))
        .with_batch_exporter(exporter)
        .with_sampler(Sampler::AlwaysOn)
        .build();

    global::set_tracer_provider(provider.clone());
    let tracer = provider.tracer("lana-tracer");
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let fmt_layer = fmt::layer().compact();

    // set env to RUST_LOG=debug or RUST_LOG=info,sqlx=warn to change this value dynamically
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(telemetry)
        .init();

    setup_panic_hook();

    Ok(())
}

fn setup_panic_hook() {
    let default_panic = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        let span = error_span!("panic", panic_type = "unhandled");
        let _guard = span.enter();

        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic payload".to_string()
        };

        error!(
            target: "panic",
            panic_message = %message,
            panic_location = ?panic_info.location(),
            panic_thread = ?std::thread::current().name(),
            panic_backtrace = ?std::backtrace::Backtrace::capture(),
            "Unhandled panic in application"
        );

        default_panic(panic_info);
    }));
}

fn telemetry_resource(config: &TracingConfig) -> Resource {
    Resource::builder()
        .with_service_name(config.service_name.clone())
        .with_attributes([KeyValue::new(SERVICE_NAMESPACE, "lana")])
        .build()
}

pub fn insert_error_fields(level: tracing::Level, error: impl std::fmt::Display) {
    Span::current().record("error", tracing::field::display("true"));
    Span::current().record("error.level", tracing::field::display(level));
    Span::current().record("error.message", tracing::field::display(error));
}

#[cfg(feature = "http")]
pub mod http {
    pub fn extract_tracing(headers: &axum_extra::headers::HeaderMap) {
        use opentelemetry::propagation::text_map_propagator::TextMapPropagator;
        use opentelemetry_http::HeaderExtractor;
        use opentelemetry_sdk::propagation::TraceContextPropagator;
        use tracing_opentelemetry::OpenTelemetrySpanExt;
        let extractor = HeaderExtractor(headers);
        let propagator = TraceContextPropagator::new();
        let ctx = propagator.extract(&extractor);
        let _ = tracing::Span::current().set_parent(ctx);
    }

    pub fn inject_trace() -> axum_extra::headers::HeaderMap {
        use opentelemetry::propagation::TextMapPropagator;
        use opentelemetry_http::HeaderInjector;
        use opentelemetry_sdk::propagation::TraceContextPropagator;
        use tracing_opentelemetry::OpenTelemetrySpanExt;

        let mut header_map = axum_extra::headers::HeaderMap::new();
        let mut header_wrapper = HeaderInjector(&mut header_map);
        let propagator = TraceContextPropagator::new();
        let context = tracing::Span::current().context();
        propagator.inject_context(&context, &mut header_wrapper);

        header_map
    }

    #[cfg(feature = "reqwest")]
    pub fn inject_trace_reqwest() -> reqwest::header::HeaderMap {
        use opentelemetry::propagation::TextMapPropagator;
        use opentelemetry_http::HeaderInjector;
        use opentelemetry_sdk::propagation::TraceContextPropagator;
        use tracing_opentelemetry::OpenTelemetrySpanExt;

        let mut header_map = reqwest::header::HeaderMap::new();
        let mut header_wrapper = HeaderInjector(&mut header_map);
        let propagator = TraceContextPropagator::new();
        let context = tracing::Span::current().context();
        propagator.inject_context(&context, &mut header_wrapper);

        header_map
    }
}

#[cfg(feature = "persistence")]
pub mod persistence {
    use serde::{Deserialize, Serialize};
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct SerializableTraceContext {
        pub traceparent: Option<String>,
        pub tracestate: Option<String>,
    }

    pub fn extract() -> SerializableTraceContext {
        use opentelemetry::propagation::TextMapPropagator;
        use opentelemetry_sdk::propagation::TraceContextPropagator;

        let mut carrier = std::collections::HashMap::new();
        let propagator = TraceContextPropagator::new();
        let current_context = tracing::Span::current().context();

        propagator.inject_context(&current_context, &mut carrier);

        SerializableTraceContext {
            traceparent: carrier.get("traceparent").cloned(),
            tracestate: carrier.get("tracestate").cloned(),
        }
    }

    pub fn set_parent(context: &SerializableTraceContext) {
        use opentelemetry::propagation::TextMapPropagator;
        use opentelemetry_sdk::propagation::TraceContextPropagator;

        let mut carrier = std::collections::HashMap::new();

        if let Some(traceparent) = &context.traceparent {
            carrier.insert("traceparent".to_string(), traceparent.clone());
        }
        if let Some(tracestate) = &context.tracestate {
            carrier.insert("tracestate".to_string(), tracestate.clone());
        }

        let propagator = TraceContextPropagator::new();
        let extracted_context = propagator.extract(&carrier);
        let _ = tracing::Span::current().set_parent(extracted_context);
    }
}
