---
id: observability
title: Observability
sidebar_position: 11
---

# Traceability and Observability

This document describes Lana's observability infrastructure, including distributed tracing with OpenTelemetry.

```mermaid
graph TD
    subgraph EntryPoints["Entry Points"]
        CLI["lana-cli"]
        AS["admin-server"]
        CS["customer-server"]
    end

    subgraph AppLayer["Application Layer"]
        LA["lana-app"]
        EVENTS["lana-events<br/>Event Definitions"]
        RBAC["Role Types<br/>RBAC Definitions"]
        IDS["lana-ids<br/>Entity ID Types"]
    end

    subgraph DomainLayer["Domain Layer"]
        CC["core-credit"]
        CD["core-deposit"]
        CCU["core-customer"]
        CA["core-accounting"]
        CCUS["core-custody"]
        COB["core-applicant"]
        GOV["governance"]
    end

    CLI --> LA
    AS --> LA
    CS --> LA
    LA --> EVENTS
    LA --> RBAC
    LA --> IDS
    LA --> CC
    LA --> CD
    LA --> CCU
    LA --> CA
    LA --> CCUS
    LA --> COB
    LA --> GOV
```

## Overview

Lana implements comprehensive observability through:

- **Distributed Tracing**: OpenTelemetry integration
- **Structured Logging**: Contextual log entries
- **Metrics**: Performance and business metrics
- **Correlation**: Request tracing across services

## Architecture

```mermaid
graph TD
    TRACES["Application<br/>(Traces)"] --> OTEL
    LOGS["Application<br/>(Logs)"] --> OTEL
    METRICS["Application<br/>(Metrics)"] --> OTEL
    OTEL["OpenTelemetry SDK<br/>(Unified Telemetry Collection)"] --> BACKEND["Telemetry Backend<br/>(Jaeger / Prometheus / Loki)"]
```

## OpenTelemetry Integration

### Configuration

```rust
use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::prelude::*;

pub fn init_telemetry() -> Result<()> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317")
        )
        .install_batch(opentelemetry::runtime::Tokio)?;

    let telemetry = tracing_opentelemetry::layer()
        .with_tracer(tracer);

    tracing_subscriber::registry()
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}
```

### Instrumentation

```rust
use tracing::{instrument, info, span, Level};

#[instrument(skip(self), fields(facility_id = %facility_id))]
pub async fn process_disbursal(
    &self,
    facility_id: CreditFacilityId,
) -> Result<Disbursal> {
    info!("Processing disbursal");

    let facility = self.repo.find_by_id(facility_id).await?;

    // Nested span for ledger operation
    let ledger_span = span!(Level::INFO, "ledger_transfer");
    let _guard = ledger_span.enter();

    self.ledger.transfer(facility.amount).await?;

    info!("Disbursal completed");
    Ok(disbursal)
}
```

## Context Propagation

### HTTP Headers

Trace context propagates through HTTP headers:

```
traceparent: 00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01
tracestate: lana=value
```

### GraphQL Context

```rust
pub struct GraphQLContext {
    trace_id: TraceId,
    span_id: SpanId,
    subject: SubjectId,
}

impl GraphQLContext {
    pub fn from_request(req: &Request) -> Self {
        let trace_context = extract_trace_context(req);
        Self {
            trace_id: trace_context.trace_id,
            span_id: trace_context.span_id,
            subject: extract_subject(req),
        }
    }
}
```

## Structured Logging

### Log Format

```rust
use tracing::{info, warn, error};

// Structured log with context
info!(
    customer_id = %customer.id,
    email = %customer.email,
    "Customer created"
);

// Error with context
error!(
    error = %e,
    facility_id = %facility_id,
    "Failed to process disbursal"
);
```

### Log Levels

| Level | Usage |
|-------|-------|
| ERROR | System failures, requires attention |
| WARN | Unexpected but recoverable conditions |
| INFO | Business operations, audit events |
| DEBUG | Detailed debugging information |
| TRACE | Very detailed tracing |

## Metrics

### Business Metrics

```rust
use metrics::{counter, gauge, histogram};

// Count operations
counter!("disbursals_processed_total").increment(1);

// Track current values
gauge!("active_facilities").set(count as f64);

// Measure durations
histogram!("disbursal_processing_seconds").record(duration);
```

### System Metrics

- Request latency
- Error rates
- Database connection pool
- Memory usage

## Correlation IDs

All requests carry correlation context:

```rust
#[derive(Debug, Clone)]
pub struct CorrelationContext {
    pub trace_id: String,
    pub span_id: String,
    pub request_id: Uuid,
    pub subject_id: Option<SubjectId>,
}

impl CorrelationContext {
    pub fn new() -> Self {
        Self {
            trace_id: Span::current().context().trace_id().to_string(),
            span_id: Span::current().context().span_id().to_string(),
            request_id: Uuid::new_v4(),
            subject_id: None,
        }
    }
}
```

## Dashboards

### Operations Dashboard

- Request volume and latency
- Error rates by endpoint
- Active users
- System health

### Business Dashboard

- Facility creations per day
- Disbursement volume
- Payment processing
- Customer onboarding

## Alerts

### Critical Alerts

- Service unavailable
- High error rate (>5%)
- Database connection failures
- External service timeouts

### Warning Alerts

- Elevated latency
- High queue depth
- Memory pressure
- Disk space low

