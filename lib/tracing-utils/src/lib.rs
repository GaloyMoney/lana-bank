#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod error;
pub mod error_severity;

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use opentelemetry::{KeyValue, global, trace::TracerProvider};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    Resource,
    propagation::TraceContextPropagator,
    trace::{BatchConfigBuilder, BatchSpanProcessor, Sampler, SdkTracerProvider},
};
use opentelemetry_semantic_conventions::resource::SERVICE_NAMESPACE;
use serde::{Deserialize, Serialize};
use tracing_subscriber::{
    Layer, filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

pub use error::TracingError;
pub use error_severity::ErrorSeverity;
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

/// Handle for managing tracer lifecycle
#[derive(Clone)]
pub struct TracerHandle {
    provider: Arc<SdkTracerProvider>,
    shutdown_called: Arc<AtomicBool>,
}

// Global handle for shutdown coordination
static TRACER_HANDLE: Mutex<Option<TracerHandle>> = Mutex::new(None);

pub fn init_tracer(config: TracingConfig) -> anyhow::Result<()> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()?;

    let batch_config = BatchConfigBuilder::default()
        .with_max_queue_size(8192)
        .build();

    let batch_processor = BatchSpanProcessor::builder(exporter)
        .with_batch_config(batch_config)
        .build();

    let provider = SdkTracerProvider::builder()
        .with_resource(telemetry_resource(&config))
        .with_span_processor(batch_processor)
        .with_sampler(Sampler::AlwaysOn)
        .build();

    let provider_arc = Arc::new(provider);

    // Store handle in global state for graceful shutdown
    let handle = TracerHandle {
        provider: Arc::clone(&provider_arc),
        shutdown_called: Arc::new(AtomicBool::new(false)),
    };
    {
        let mut guard = TRACER_HANDLE.lock().expect("Failed to lock tracer handle");
        *guard = Some(handle);
    }

    global::set_tracer_provider((*provider_arc).clone());
    let tracer = provider_arc.tracer("lana-tracer");

    // Build separate filter for OTEL that excludes tokio/runtime
    let otel_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info,sqlx=debug"))
        .expect("default EnvFilter should be valid")
        .add_directive("tokio=off".parse().expect("tokio=off directive is valid"))
        .add_directive(
            "runtime=off"
                .parse()
                .expect("runtime=off directive is valid"),
        );

    let telemetry = tracing_opentelemetry::layer()
        .with_tracer(tracer)
        .with_filter(otel_filter);

    // Build separate filter for fmt_layer that excludes tokio/runtime/sqlx from stdout
    let fmt_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .expect("default EnvFilter should be valid")
        .add_directive("tokio=off".parse().expect("tokio=off directive is valid"))
        .add_directive(
            "runtime=off"
                .parse()
                .expect("runtime=off directive is valid"),
        )
        .add_directive("sqlx=off".parse().expect("sqlx=off directive is valid"));

    let fmt_layer = fmt::layer().compact().with_filter(fmt_filter);

    let base_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info,sqlx=debug"))
        .expect("default EnvFilter should be valid");

    tracing_subscriber::registry()
        .with(base_filter)
        .with(fmt_layer)
        .with(telemetry)
        .init();

    setup_panic_hook();

    Ok(())
}

/// Gracefully shutdown the tracer provider, flushing all pending spans
///
/// This should be called during application shutdown to ensure all telemetry
/// data is properly exported before the process exits.
pub fn shutdown_tracer() -> Result<(), TracingError> {
    // Get the handle from global state
    let handle = {
        let guard = TRACER_HANDLE.lock().expect("Failed to lock tracer handle");
        guard.clone()
    };

    if let Some(handle) = handle {
        perform_shutdown(handle)?;
    } else {
        eprintln!("No tracer handle found during shutdown");
    }

    Ok(())
}

fn perform_shutdown(handle: TracerHandle) -> Result<(), TracingError> {
    // Ensure shutdown is only called once using atomic bool
    if handle
        .shutdown_called
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        eprintln!("Tracer shutdown already called, skipping");
        return Ok(());
    }

    println!("Shutting down tracer provider");

    // Force flush and shutdown the provider
    // This ensures all pending spans are exported
    if let Err(e) = handle.provider.force_flush() {
        eprintln!("Error flushing tracer provider: {:?}", e);
    } else {
        println!("Tracer provider flushed successfully");
    }

    handle.provider.shutdown()?;
    println!("Tracer provider shut down successfully");

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

#[cfg(feature = "http")]
pub mod http {
    use axum_extra::headers::HeaderMap;
    use opentelemetry::propagation::text_map_propagator::TextMapPropagator;
    use opentelemetry_http::HeaderExtractor;
    use opentelemetry_sdk::propagation::TraceContextPropagator;
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    /// Middleware layer that extracts trace context from incoming HTTP headers
    /// and makes it the parent of all subsequent spans in the request.
    pub async fn trace_context_middleware(
        headers: HeaderMap,
        request: axum::extract::Request,
        next: axum::middleware::Next,
    ) -> axum::response::Response {
        use tracing::Instrument;

        let extractor = HeaderExtractor(&headers);
        let propagator = TraceContextPropagator::new();
        let parent_ctx = propagator.extract(&extractor);

        // Create a span that will be the parent for all handler spans
        let span = tracing::info_span!("http_request");
        let _ = span.set_parent(parent_ctx);

        // Execute the rest of the request within this span
        next.run(request).instrument(span).await
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
