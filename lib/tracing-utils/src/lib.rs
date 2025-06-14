#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

use opentelemetry::{KeyValue, global, trace::TracerProvider};
use opentelemetry_sdk::{
    Resource,
    propagation::TraceContextPropagator,
    resource::{EnvResourceDetector, SdkProvidedResourceDetector},
    trace::{Config, Sampler},
};
use opentelemetry_semantic_conventions::resource::{SERVICE_NAME, SERVICE_NAMESPACE};
use serde::{Deserialize, Serialize};
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub use tracing::*;

use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    let provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_trace_config(
            Config::default()
                .with_resource(telemetry_resource(&config))
                .with_sampler(Sampler::AlwaysOn),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;
    let telemetry =
        tracing_opentelemetry::layer().with_tracer(provider.tracer(config.service_name));

    let fmt_layer = fmt::layer().json();
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info,otel::tracing=trace,sqlx=warn,sqlx_ledger=info"))
        .unwrap();
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(telemetry)
        .init();

    Ok(())
}

fn telemetry_resource(config: &TracingConfig) -> Resource {
    Resource::from_detectors(
        Duration::from_secs(3),
        vec![
            Box::new(EnvResourceDetector::new()),
            Box::new(SdkProvidedResourceDetector),
        ],
    )
    .merge(&Resource::new(vec![
        KeyValue::new(SERVICE_NAME, config.service_name.clone()),
        KeyValue::new(SERVICE_NAMESPACE, "lana"),
    ]))
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
        tracing::Span::current().set_parent(ctx)
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
        tracing::Span::current().set_parent(extracted_context);
    }
}
