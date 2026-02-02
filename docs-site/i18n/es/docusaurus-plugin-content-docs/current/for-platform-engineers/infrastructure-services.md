---
id: infrastructure-services
title: Servicios de Infraestructura
sidebar_position: 11
---

# Servicios de Infraestructura

Este documento describe los servicios de infraestructura compartidos que soportan los servicios de dominio en Lana Bank. Estos servicios proporcionan capacidades transversales como auditoría, autorización, publicación de eventos y trazabilidad.

## Visión General de la Arquitectura

```
┌─────────────────────────────────────────────────────────────────┐
│                    Servicios de Dominio                         │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐               │
│  │core-credit  │ │core-deposit │ │core-customer│               │
│  └─────────────┘ └─────────────┘ └─────────────┘               │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                Servicios de Infraestructura (lib/*)             │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐            │
│  │  audit  │  │  authz  │  │  outbox │  │   job   │            │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘            │
│  ┌─────────────────┐  ┌─────────────────────────────┐          │
│  │  tracing-utils  │  │     cloud-storage           │          │
│  └─────────────────┘  └─────────────────────────────┘          │
└─────────────────────────────────────────────────────────────────┘
```

## Componentes Principales

### Sistema de Auditoría (audit)

Proporciona registro inmutable de acciones para cumplimiento normativo y trazabilidad.

```rust
// lib/audit/src/lib.rs
pub struct AuditService {
    pool: PgPool,
    tracer: Tracer,
}

impl AuditService {
    pub async fn record(
        &self,
        subject: &Subject,
        action: Action,
        object: Object,
        outcome: Outcome,
    ) -> Result<AuditEntry, AuditError> {
        let entry = AuditEntry {
            id: AuditEntryId::new(),
            subject_id: subject.id().to_string(),
            subject_type: subject.subject_type(),
            action: action.to_string(),
            object_type: object.object_type(),
            object_id: object.id().map(|id| id.to_string()),
            outcome: outcome.to_string(),
            metadata: serde_json::Value::Null,
            trace_id: self.current_trace_id(),
            created_at: Utc::now(),
        };

        self.persist(&entry).await?;
        Ok(entry)
    }
}
```

#### Estructura de Entrada de Auditoría

| Campo | Tipo | Descripción |
|-------|------|-------------|
| id | UUID | Identificador único |
| subject_id | String | ID del actor (usuario/sistema) |
| subject_type | String | Tipo de actor |
| action | String | Acción realizada |
| object_type | String | Tipo de recurso afectado |
| object_id | String | ID del recurso |
| outcome | String | Resultado (success/failure) |
| trace_id | String | ID de traza para correlación |
| created_at | Timestamp | Fecha y hora |

### Sistema de Autorización (authz)

Implementa Control de Acceso Basado en Roles (RBAC) usando Casbin.

```rust
// lib/authz/src/lib.rs
pub struct AuthzService {
    enforcer: Arc<RwLock<Enforcer>>,
}

impl AuthzService {
    pub async fn enforce(
        &self,
        subject: &Subject,
        object: Object,
        action: Action,
    ) -> Result<(), AuthzError> {
        let enforcer = self.enforcer.read().await;

        let allowed = enforcer.enforce((
            subject.id().to_string(),
            object.to_string(),
            action.to_string(),
        ))?;

        if !allowed {
            return Err(AuthzError::PermissionDenied {
                subject: subject.id().to_string(),
                object: object.to_string(),
                action: action.to_string(),
            });
        }

        Ok(())
    }

    pub async fn add_role_for_user(
        &self,
        user_id: &str,
        role: &str,
    ) -> Result<(), AuthzError> {
        let mut enforcer = self.enforcer.write().await;
        enforcer.add_role_for_user(user_id, role, None).await?;
        Ok(())
    }
}
```

#### Modelo Casbin

```conf
# model.conf
[request_definition]
r = sub, obj, act

[policy_definition]
p = sub, obj, act

[role_definition]
g = _, _

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = g(r.sub, p.sub) && r.obj == p.obj && r.act == p.act
```

### Publicación de Eventos (outbox)

Implementa el patrón outbox para entrega confiable de eventos.

```rust
// lib/outbox/src/lib.rs
pub struct OutboxPublisher {
    pool: PgPool,
}

impl OutboxPublisher {
    pub async fn publish<E: Serialize>(
        &self,
        event: &E,
        db_op: &mut DbOp<'_>,
    ) -> Result<i64, OutboxError> {
        let payload = serde_json::to_value(event)?;
        let trace_context = TraceContext::from_current_span();

        let sequence = sqlx::query_scalar!(
            r#"
            INSERT INTO outbox_events (event_type, payload, trace_context)
            VALUES ($1, $2, $3)
            RETURNING sequence
            "#,
            std::any::type_name::<E>(),
            payload,
            serde_json::to_value(&trace_context)?,
        )
        .fetch_one(db_op.as_mut())
        .await?;

        Ok(sequence)
    }
}
```

### Sistema de Trabajos (job)

Proporciona infraestructura para procesamiento de tareas en segundo plano.

```rust
// lib/job/src/lib.rs
pub trait Job: Send + Sync + 'static {
    const NAME: &'static str;

    async fn run(&self, current_job: CurrentJob) -> Result<JobCompletion, JobError>;
}

pub enum JobCompletion {
    Complete,
    RescheduleAt(DateTime<Utc>),
    RescheduleIn(Duration),
}

pub struct JobRegistry {
    jobs: HashMap<String, Arc<dyn Job>>,
}

impl JobRegistry {
    pub fn register<J: Job>(&mut self, job: J) {
        self.jobs.insert(J::NAME.to_string(), Arc::new(job));
    }
}
```

### Trazado y Observabilidad (tracing-utils)

Proporciona integración con OpenTelemetry para trazabilidad distribuida.

```rust
// lib/tracing-utils/src/lib.rs
pub fn init_tracer(config: TracingConfig) -> Result<(), TracingError> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(&config.otlp_endpoint),
        )
        .with_trace_config(
            opentelemetry::sdk::trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", config.service_name),
                ])),
        )
        .install_batch(opentelemetry::runtime::Tokio)?;

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}
```

### Almacenamiento en la Nube (cloud-storage)

Abstracción para almacenamiento de archivos (documentos, reportes).

```rust
// lib/cloud-storage/src/lib.rs
#[async_trait]
pub trait StorageProvider: Send + Sync {
    async fn upload(&self, key: &str, data: &[u8]) -> Result<StorageUrl, StorageError>;
    async fn download(&self, key: &str) -> Result<Vec<u8>, StorageError>;
    async fn delete(&self, key: &str) -> Result<(), StorageError>;
    async fn get_signed_url(&self, key: &str, expiry: Duration) -> Result<String, StorageError>;
}

pub struct GcsProvider {
    client: cloud_storage::Client,
    bucket: String,
}

pub struct S3Provider {
    client: aws_sdk_s3::Client,
    bucket: String,
}
```

## Patrones de Integración

### Patrón de Uso Típico

```rust
// En un servicio de dominio
impl CreditFacilities {
    pub async fn create_facility(
        &self,
        subject: &Subject,
        input: CreateFacilityInput,
    ) -> Result<CreditFacility, Error> {
        // 1. Autorización
        self.authz.enforce(subject, Object::CreditFacility, Action::Create).await?;

        // 2. Lógica de negocio
        let facility = CreditFacility::new(input)?;

        // 3. Persistencia
        let mut db_op = self.pool.begin().await?;
        self.repo.create(&facility, &mut db_op).await?;

        // 4. Publicar eventos
        self.publisher.publish(&facility.events(), &mut db_op).await?;

        // 5. Registrar auditoría
        self.audit.record(
            subject,
            Action::Create,
            Object::CreditFacility(facility.id),
            Outcome::Success,
        ).await?;

        db_op.commit().await?;
        Ok(facility)
    }
}
```

### Gráfico de Dependencias

```
┌─────────────────────────────────────────────────────────────────┐
│                         lana-app                                │
└─────────────────────────────────────────────────────────────────┘
         │
         ├──────────────────────────────────────────────────┐
         ▼                                                  ▼
┌─────────────────┐                              ┌─────────────────┐
│  core-credit    │──────────┐                   │  core-deposit   │
└─────────────────┘          │                   └─────────────────┘
         │                   │                            │
         │                   ▼                            │
         │          ┌─────────────────┐                   │
         │          │  core-accounting│                   │
         │          └─────────────────┘                   │
         │                   │                            │
         └───────────────────┼────────────────────────────┘
                             │
                             ▼
         ┌───────────────────┼───────────────────┐
         │                   │                   │
         ▼                   ▼                   ▼
    ┌─────────┐        ┌─────────┐        ┌─────────┐
    │  audit  │        │  authz  │        │  outbox │
    └─────────┘        └─────────┘        └─────────┘
         │                   │                   │
         └───────────────────┼───────────────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │   PostgreSQL    │
                    └─────────────────┘
```

## Responsabilidades de Componentes

| Componente | Responsabilidad | Dependencias |
|------------|-----------------|--------------|
| audit | Registro inmutable de acciones | PostgreSQL, tracing-utils |
| authz | Control de acceso RBAC | PostgreSQL (políticas Casbin) |
| outbox | Publicación confiable de eventos | PostgreSQL |
| job | Procesamiento en segundo plano | PostgreSQL, tracing-utils |
| tracing-utils | Trazabilidad distribuida | OpenTelemetry |
| cloud-storage | Almacenamiento de archivos | GCS/S3 |

## Configuración y Features

### Feature Flags

```toml
# lib/audit/Cargo.toml
[features]
default = []
graphql = ["async-graphql"]

# lib/authz/Cargo.toml
[features]
default = []
postgres-adapter = ["sqlx"]

# lib/outbox/Cargo.toml
[features]
default = []
test-helpers = []
```

### Variables de Entorno

| Variable | Servicio | Propósito |
|----------|----------|-----------|
| DATABASE_URL | Todos | Conexión PostgreSQL |
| OTEL_EXPORTER_OTLP_ENDPOINT | tracing-utils | Endpoint OTEL |
| GCS_BUCKET | cloud-storage | Bucket de GCS |
| AWS_S3_BUCKET | cloud-storage | Bucket de S3 |
