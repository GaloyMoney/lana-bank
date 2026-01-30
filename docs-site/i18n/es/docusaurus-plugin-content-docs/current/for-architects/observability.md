---
id: observability
title: Trazabilidad y Observabilidad
sidebar_position: 8
---

# Trazabilidad y Observabilidad

Este documento describe la infraestructura de trazabilidad distribuida y observabilidad del sistema Lana Bank. Cubre la integración con OpenTelemetry, la propagación del contexto de traza a través de límites de servicio y trabajos asíncronos, patrones de instrumentación y funcionalidades de observabilidad.

![Arquitectura de Observabilidad](/img/architecture/observability-1.png)

## Arquitectura de Observabilidad

El sistema implementa trazabilidad distribuida usando OpenTelemetry (OTEL) para proporcionar observabilidad de extremo a extremo.

```
┌─────────────────────────────────────────────────────────────────┐
│                     Servicios de Aplicación                     │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │  admin-server   │  │ customer-server │  │  Background     │ │
│  │                 │  │                 │  │     Jobs        │ │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘ │
│           │                    │                    │          │
│           └────────────────────┼────────────────────┘          │
│                                │                               │
│                    ┌───────────▼───────────┐                   │
│                    │    tracing-utils      │                   │
│                    │   (Librería central)  │                   │
│                    └───────────┬───────────┘                   │
└────────────────────────────────┼───────────────────────────────┘
                                 │
                    ┌────────────▼────────────┐
                    │    OTEL Collector       │
                    │   (otel-agent:4317)     │
                    └────────────┬────────────┘
                                 │
           ┌─────────────────────┼─────────────────────┐
           │                     │                     │
           ▼                     ▼                     ▼
    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
    │    Jaeger    │    │  Prometheus  │    │   Grafana    │
    │   (Trazas)   │    │  (Métricas)  │    │  (Dashboards)│
    └──────────────┘    └──────────────┘    └──────────────┘
```

## Integración con OpenTelemetry

### Inicialización del Tracer

La librería `tracing-utils` proporciona la función `init_tracer()` que configura la canalización completa de trazas:

```rust
// lib/tracing-utils/src/lib.rs
pub fn init_tracer(config: TracingConfig) -> Result<(), TracingError> {
    // Configurar propagador W3C
    opentelemetry::global::set_text_map_propagator(
        TraceContextPropagator::new()
    );

    // Configurar exportador OTLP
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(&config.otlp_endpoint)
        .build_span_exporter()?;

    // Configurar proveedor de trazas
    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_sampler(Sampler::AlwaysOn)
        .with_resource(Resource::new(vec![
            KeyValue::new("service.name", config.service_name),
            KeyValue::new("service.version", config.service_version),
        ]))
        .build();

    opentelemetry::global::set_tracer_provider(provider);

    // Configurar subscriber de tracing
    let telemetry = tracing_opentelemetry::layer()
        .with_tracer(opentelemetry::global::tracer("lana"));

    let subscriber = Registry::default()
        .with(EnvFilter::from_default_env())
        .with(telemetry)
        .with(fmt::layer());

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
```

### Componentes Clave

| Componente | Propósito | Configuración |
|------------|-----------|---------------|
| TraceContextPropagator | Propagación de contexto W3C | Propagador global |
| SpanExporter | Exportador OTLP sobre gRPC | Variable `OTEL_EXPORTER_OTLP_ENDPOINT` |
| TracerProvider | Proveedor de trazas | Exportador por lotes, Sampler AlwaysOn |
| tracing-opentelemetry | Puente al ecosistema tracing | Capa de telemetría |
| EnvFilter | Filtrado de nivel de logs | Variable `RUST_LOG` |

### Configuración del Servicio

```rust
pub struct TracingConfig {
    pub service_name: String,
    pub service_version: String,
    pub otlp_endpoint: String,
}

impl TracingConfig {
    pub fn from_env() -> Self {
        Self {
            service_name: std::env::var("OTEL_SERVICE_NAME")
                .unwrap_or_else(|_| "lana".to_string()),
            service_version: std::env::var("OTEL_SERVICE_VERSION")
                .unwrap_or_else(|_| "0.0.0".to_string()),
            otlp_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:4317".to_string()),
        }
    }
}
```

## Propagación del Contexto de Traza

### Propagación HTTP

#### Extracción de Solicitudes Entrantes

```rust
pub fn extract_trace_context(headers: &HeaderMap) -> Context {
    let extractor = HeaderExtractor(headers);
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&extractor)
    })
}

// Uso en handler GraphQL
async fn graphql_handler(headers: HeaderMap, req: Request) -> Response {
    let parent_context = extract_trace_context(&headers);

    let span = tracing::info_span!("graphql.request");
    span.set_parent(parent_context);

    span.in_scope(|| {
        // Procesar solicitud
    }).await
}
```

#### Inyección en Solicitudes Salientes

```rust
pub fn inject_trace_context(headers: &mut HeaderMap) {
    let mut injector = HeaderInjector(headers);
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&Span::current().context(), &mut injector)
    });
}

// Uso en cliente HTTP
async fn call_external_service(&self, url: &str) -> Result<Response, Error> {
    let mut headers = HeaderMap::new();
    inject_trace_context(&mut headers);

    self.client
        .get(url)
        .headers(headers)
        .send()
        .await
}
```

### Propagación Basada en Persistencia

Para trabajos asíncronos, el contexto se serializa y almacena:

```rust
#[derive(Serialize, Deserialize)]
pub struct SerializedTraceContext {
    pub traceparent: String,
    pub tracestate: Option<String>,
}

impl SerializedTraceContext {
    pub fn from_current() -> Self {
        let span = Span::current();
        let context = span.context();

        Self {
            traceparent: extract_traceparent(&context),
            tracestate: extract_tracestate(&context),
        }
    }

    pub fn restore(&self) -> Context {
        let mut carrier = HashMap::new();
        carrier.insert("traceparent".to_string(), self.traceparent.clone());
        if let Some(state) = &self.tracestate {
            carrier.insert("tracestate".to_string(), state.clone());
        }

        opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.extract(&carrier)
        })
    }
}
```

## Patrones de Instrumentación

### La Macro #[instrument]

```rust
use tracing::instrument;

#[instrument(
    name = "credit_facility.create",
    skip(self, input),
    fields(customer_id = %input.customer_id)
)]
pub async fn create_facility(
    &self,
    input: CreateFacilityInput,
) -> Result<CreditFacility, Error> {
    // Lógica de negocio...
}
```

### Creación Manual de Spans

```rust
use tracing::{info_span, Instrument};

pub async fn process_payment(&self, payment_id: PaymentId) -> Result<(), Error> {
    let span = info_span!(
        "payment.process",
        payment.id = %payment_id,
        payment.status = tracing::field::Empty
    );

    async {
        let payment = self.load_payment(payment_id).await?;

        // Registrar campo dinámico
        Span::current().record("payment.status", &payment.status.to_string());

        self.execute_payment(&payment).await
    }
    .instrument(span)
    .await
}
```

### Registro de Campos Adicionales

```rust
use tracing::Span;

pub async fn disburse(&self, facility_id: CreditFacilityId, amount: Money) {
    // Registrar información adicional en el span actual
    Span::current().record("facility.id", &facility_id.to_string());
    Span::current().record("amount", &amount.to_string());

    // También se pueden registrar eventos
    tracing::info!(
        facility_id = %facility_id,
        amount = %amount,
        "Disbursal initiated"
    );
}
```

## Funcionalidades de Observabilidad

### Hook de Captura de Pánicos

```rust
pub fn setup_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        let span = Span::current();

        // Registrar información del pánico en el span
        span.record("panic", &true);
        span.record("panic.message", &panic_info.to_string());

        if let Some(location) = panic_info.location() {
            span.record("panic.file", location.file());
            span.record("panic.line", &location.line());
        }

        tracing::error!(
            panic = true,
            message = %panic_info,
            "Application panic"
        );
    }));
}
```

### Registro de Campos de Error

```rust
pub fn record_error(error: &impl std::error::Error) {
    Span::current().record("error", &true);
    Span::current().record("error.message", &error.to_string());

    if let Some(source) = error.source() {
        Span::current().record("error.source", &source.to_string());
    }

    tracing::error!(
        error = %error,
        "Operation failed"
    );
}
```

## Configuración

### Variables de Entorno

| Variable | Propósito | Valor por defecto |
|----------|-----------|-------------------|
| `OTEL_EXPORTER_OTLP_ENDPOINT` | Endpoint del colector OTEL | `http://localhost:4317` |
| `OTEL_SERVICE_NAME` | Nombre del servicio | `lana` |
| `OTEL_SERVICE_VERSION` | Versión del servicio | `0.0.0` |
| `RUST_LOG` | Nivel de logging | `info` |

### Configuración del Colector OTEL

```yaml
# dev/otel-agent-config.yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    timeout: 1s
    send_batch_size: 1024

exporters:
  jaeger:
    endpoint: jaeger:14250
    tls:
      insecure: true

  prometheus:
    endpoint: 0.0.0.0:8889

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters: [jaeger]

    metrics:
      receivers: [otlp]
      processors: [batch]
      exporters: [prometheus]
```

## Integración en Servicios

### Servidores GraphQL

```rust
// lana/admin-server/src/lib.rs
pub async fn run(config: ServerConfig, app: Arc<LanaApp>) -> Result<(), Error> {
    // Inicializar trazado
    init_tracer(TracingConfig {
        service_name: "admin-server".to_string(),
        service_version: env!("CARGO_PKG_VERSION").to_string(),
        otlp_endpoint: config.otlp_endpoint.clone(),
    })?;

    // Configurar router con middleware de trazado
    let app = Router::new()
        .route("/graphql", post(graphql_handler))
        .layer(TraceLayer::new_for_http());

    axum::Server::bind(&config.bind_address)
        .serve(app.into_make_service())
        .await
}
```

### Sistema de Trabajos

```rust
// lib/job/src/dispatcher.rs
impl JobDispatcher {
    async fn execute_job(&self, job: CurrentJob) {
        // Restaurar contexto de traza
        let parent_context = job.trace_context
            .as_ref()
            .map(|tc| tc.restore())
            .unwrap_or_else(Context::current);

        let span = info_span!(
            "job.execute",
            job.type = %job.job_type,
            job.id = %job.id,
            job.attempt = job.attempt_index
        );
        span.set_parent(parent_context);

        span.in_scope(|| {
            self.runner.run(job)
        }).await
    }
}
```

## Desarrollo Local

En desarrollo con Tilt, las trazas se pueden visualizar en Jaeger:

```bash
# Jaeger UI disponible en
http://localhost:16686

# Prometheus métricas en
http://localhost:9090
```
