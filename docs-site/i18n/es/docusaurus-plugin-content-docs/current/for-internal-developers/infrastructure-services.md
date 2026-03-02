---
id: infrastructure-services
title: Servicios de Infraestructura
sidebar_position: 10
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
