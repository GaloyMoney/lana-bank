---
id: audit-system
title: Sistema de Auditoría
sidebar_position: 12
---

# Sistema de Auditoría y Registro

Este documento describe el sistema de auditoría y registro implementado en Lana Bank para cumplimiento normativo y trazabilidad de operaciones.

## Visión General del Sistema

El sistema de auditoría proporciona:

- **Registro inmutable**: Todas las acciones de negocio se registran permanentemente
- **Trazabilidad completa**: Correlación entre operaciones mediante trace IDs
- **Cumplimiento normativo**: Soporte para requisitos regulatorios bancarios
- **Consulta y análisis**: API GraphQL para consulta de registros

## Arquitectura de la Traza de Auditoría

```
┌─────────────────────────────────────────────────────────────────┐
│                    Servicios de Dominio                         │
│  ┌─────────────────┐  ┌─────────────────┐                      │
│  │   core-credit   │  │   core-deposit  │                      │
│  └────────┬────────┘  └────────┬────────┘                      │
│           │                    │                               │
│           └────────────────────┘                               │
│                       │                                        │
│                       ▼                                        │
│           ┌───────────────────────┐                            │
│           │    AuditService       │                            │
│           │    record()           │                            │
│           └───────────┬───────────┘                            │
└───────────────────────┼────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────────┐
│                     PostgreSQL                                  │
│           ┌───────────────────────┐                            │
│           │    audit_entries      │                            │
│           └───────────────────────┘                            │
└─────────────────────────────────────────────────────────────────┘
```

## Estructura de la Entrada de Auditoría

```rust
pub struct AuditEntry {
    pub id: AuditEntryId,
    pub subject_id: String,
    pub subject_type: SubjectType,
    pub action: String,
    pub object_type: String,
    pub object_id: Option<String>,
    pub outcome: Outcome,
    pub metadata: serde_json::Value,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub enum SubjectType {
    User,
    System,
    ApiKey,
}

pub enum Outcome {
    Success,
    Failure,
    PermissionDenied,
}
```

### Esquema de Base de Datos

```sql
CREATE TABLE audit_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id VARCHAR(255) NOT NULL,
    subject_type VARCHAR(50) NOT NULL,
    action VARCHAR(100) NOT NULL,
    object_type VARCHAR(100) NOT NULL,
    object_id VARCHAR(255),
    outcome VARCHAR(50) NOT NULL,
    metadata JSONB DEFAULT '{}',
    trace_id VARCHAR(32),
    span_id VARCHAR(16),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_entries_subject ON audit_entries(subject_id);
CREATE INDEX idx_audit_entries_object ON audit_entries(object_type, object_id);
CREATE INDEX idx_audit_entries_created_at ON audit_entries(created_at DESC);
CREATE INDEX idx_audit_entries_trace_id ON audit_entries(trace_id);
```

## Marco de Registro Estructurado

### Integración con Tracing

```rust
use tracing::{info, instrument, Span};

impl AuditService {
    #[instrument(skip(self, subject, metadata), fields(
        audit.subject_id = %subject.id(),
        audit.action = %action,
        audit.object_type = %object.object_type()
    ))]
    pub async fn record(
        &self,
        subject: &Subject,
        action: Action,
        object: Object,
        outcome: Outcome,
        metadata: Option<serde_json::Value>,
    ) -> Result<AuditEntry, AuditError> {
        // Obtener IDs de traza del span actual
        let trace_id = Span::current()
            .context()
            .span()
            .span_context()
            .trace_id()
            .to_string();

        let span_id = Span::current()
            .context()
            .span()
            .span_context()
            .span_id()
            .to_string();

        let entry = AuditEntry {
            id: AuditEntryId::new(),
            subject_id: subject.id().to_string(),
            subject_type: subject.subject_type(),
            action: action.to_string(),
            object_type: object.object_type(),
            object_id: object.id().map(|id| id.to_string()),
            outcome,
            metadata: metadata.unwrap_or(serde_json::Value::Null),
            trace_id: Some(trace_id),
            span_id: Some(span_id),
            created_at: Utc::now(),
        };

        // Log estructurado
        info!(
            subject_id = %entry.subject_id,
            action = %entry.action,
            object_type = %entry.object_type,
            outcome = ?entry.outcome,
            "Audit entry recorded"
        );

        self.persist(&entry).await?;
        Ok(entry)
    }
}
```

### Infraestructura de Registro para Pruebas

```rust
#[cfg(test)]
pub fn init_test_logging() {
    use tracing_subscriber::{fmt, EnvFilter};

    let _ = fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_test_writer()
        .try_init();
}
```

## API de Auditoría GraphQL

### Estructura de la Consulta

```graphql
type Query {
    auditEntries(
        first: Int
        after: String
        filter: AuditEntryFilter
    ): AuditEntryConnection!

    auditEntry(id: ID!): AuditEntry
}

input AuditEntryFilter {
    subjectId: String
    objectType: String
    objectId: String
    action: String
    outcome: Outcome
    fromDate: DateTime
    toDate: DateTime
    traceId: String
}

type AuditEntry {
    id: ID!
    subjectId: String!
    subjectType: SubjectType!
    action: String!
    objectType: String!
    objectId: String
    outcome: Outcome!
    metadata: JSON
    traceId: String
    createdAt: DateTime!
}

type AuditEntryConnection {
    edges: [AuditEntryEdge!]!
    pageInfo: PageInfo!
}
```

### Implementación del Resolver

```rust
#[Object]
impl AuditQuery {
    async fn audit_entries(
        &self,
        ctx: &Context<'_>,
        first: Option<i32>,
        after: Option<String>,
        filter: Option<AuditEntryFilter>,
    ) -> Result<Connection<String, AuditEntry>> {
        // Verificar permisos de auditoría
        let auth = ctx.data::<AuthContext>()?;
        auth.enforce(Object::AuditEntry, Action::Read).await?;

        let audit_service = ctx.data::<AuditService>()?;
        let entries = audit_service.list(filter, first, after).await?;

        Ok(entries.into())
    }
}
```

## Interfaz Administrativa

### Componentes de la UI de Registros de Auditoría

```tsx
// apps/admin-panel/app/audit/page.tsx
export default function AuditLogsPage() {
    const { data, loading, fetchMore } = useAuditEntriesQuery({
        variables: { first: 50 }
    });

    return (
        <div className="p-6">
            <h1 className="text-2xl font-bold mb-4">Registros de Auditoría</h1>

            <AuditFilters onFilter={handleFilter} />

            <AuditTable
                entries={data?.auditEntries.edges}
                loading={loading}
            />

            <Pagination
                pageInfo={data?.auditEntries.pageInfo}
                onLoadMore={() => fetchMore({ /* ... */ })}
            />
        </div>
    );
}
```

### Componente de Tabla

```tsx
function AuditTable({ entries, loading }) {
    return (
        <Table>
            <TableHeader>
                <TableRow>
                    <TableHead>Fecha</TableHead>
                    <TableHead>Usuario</TableHead>
                    <TableHead>Acción</TableHead>
                    <TableHead>Recurso</TableHead>
                    <TableHead>Resultado</TableHead>
                </TableRow>
            </TableHeader>
            <TableBody>
                {entries?.map((edge) => (
                    <TableRow key={edge.node.id}>
                        <TableCell>
                            <DateWithTooltip date={edge.node.createdAt} />
                        </TableCell>
                        <TableCell>{edge.node.subjectId}</TableCell>
                        <TableCell>{edge.node.action}</TableCell>
                        <TableCell>
                            {edge.node.objectType}
                            {edge.node.objectId && `: ${edge.node.objectId}`}
                        </TableCell>
                        <TableCell>
                            <OutcomeBadge outcome={edge.node.outcome} />
                        </TableCell>
                    </TableRow>
                ))}
            </TableBody>
        </Table>
    );
}
```

## Cumplimiento y Pruebas

### Patrones de Verificación de Pruebas

```rust
#[tokio::test]
async fn test_audit_entry_created_on_facility_creation() {
    let app = test_app().await;

    // Crear facilidad
    let facility = app.credit_facilities
        .create(test_subject(), test_input())
        .await
        .unwrap();

    // Verificar entrada de auditoría
    let entries = app.audit
        .list_by_object("CreditFacility", &facility.id.to_string())
        .await
        .unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].action, "Create");
    assert_eq!(entries[0].outcome, Outcome::Success);
}

#[tokio::test]
async fn test_audit_entry_on_permission_denied() {
    let app = test_app().await;
    let unauthorized_subject = Subject::user("unauthorized-user");

    // Intentar operación sin permisos
    let result = app.credit_facilities
        .create(unauthorized_subject, test_input())
        .await;

    assert!(result.is_err());

    // Verificar entrada de auditoría de fallo
    let entries = app.audit
        .list_by_subject("unauthorized-user")
        .await
        .unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].outcome, Outcome::PermissionDenied);
}
```

## Análisis de Registros y Monitoreo

### Coincidencia de Patrones en Registros

```rust
pub struct AuditAnalytics {
    pool: PgPool,
}

impl AuditAnalytics {
    /// Obtener conteo de acciones por tipo en un período
    pub async fn actions_by_type(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<ActionCount>, Error> {
        sqlx::query_as!(
            ActionCount,
            r#"
            SELECT action, COUNT(*) as count
            FROM audit_entries
            WHERE created_at BETWEEN $1 AND $2
            GROUP BY action
            ORDER BY count DESC
            "#,
            from,
            to
        )
        .fetch_all(&self.pool)
        .await
    }

    /// Obtener usuarios más activos
    pub async fn most_active_users(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        limit: i32,
    ) -> Result<Vec<UserActivity>, Error> {
        sqlx::query_as!(
            UserActivity,
            r#"
            SELECT subject_id, COUNT(*) as action_count
            FROM audit_entries
            WHERE created_at BETWEEN $1 AND $2
              AND subject_type = 'User'
            GROUP BY subject_id
            ORDER BY action_count DESC
            LIMIT $3
            "#,
            from,
            to,
            limit
        )
        .fetch_all(&self.pool)
        .await
    }

    /// Detectar patrones anómalos
    pub async fn detect_anomalies(
        &self,
        threshold: i64,
    ) -> Result<Vec<AnomalyReport>, Error> {
        sqlx::query_as!(
            AnomalyReport,
            r#"
            SELECT subject_id, action, COUNT(*) as count
            FROM audit_entries
            WHERE created_at > NOW() - INTERVAL '1 hour'
            GROUP BY subject_id, action
            HAVING COUNT(*) > $1
            "#,
            threshold
        )
        .fetch_all(&self.pool)
        .await
    }
}
```

## Retención y Archivado

### Política de Retención

```sql
-- Archivar entradas antiguas (ejecutar periódicamente)
INSERT INTO audit_entries_archive
SELECT * FROM audit_entries
WHERE created_at < NOW() - INTERVAL '2 years';

DELETE FROM audit_entries
WHERE created_at < NOW() - INTERVAL '2 years';
```

### Configuración

```yaml
audit:
  retention_days: 730  # 2 años
  archive_enabled: true
  archive_destination: "gs://lana-audit-archive"
```
