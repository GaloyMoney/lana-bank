---
id: background-jobs
title: Trabajos en Segundo Plano
sidebar_position: 7
---

# Sistema de Trabajos en Segundo Plano

Este documento describe el sistema de procesamiento de trabajos en segundo plano en Lana Bank, que proporciona ejecución confiable de tareas asíncronas con lógica de reintentos, control de concurrencia y compatibilidad con trazado distribuido.

![Arquitectura de Trabajos en Segundo Plano](/img/architecture/background-jobs-1.png)

## Propósito

El sistema de trabajos habilita:
- Programación basada en tiempo (cron-like)
- Procesamiento impulsado por eventos
- Operaciones de larga duración independientes del ciclo solicitud-respuesta

## Arquitectura del Sistema

El sistema sigue una arquitectura basada en 'pull', donde un despachador central consulta trabajos pendientes en la tabla `job_executions` de PostgreSQL.

```
┌─────────────────────────────────────────────────────────────────┐
│                  Fuentes de Creación de Trabajos                │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐       │
│  │  Trabajos     │  │  Trabajos     │  │  Trabajos     │       │
│  │  Programados  │  │  por Eventos  │  │  de Sondeo    │       │
│  │  (tipo cron)  │  │  (outbox)     │  │  (auto-rep.)  │       │
│  └───────────────┘  └───────────────┘  └───────────────┘       │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Infraestructura de Trabajos                    │
│  ┌───────────────────────────────────────────────────────┐     │
│  │              JobTracker                                │     │
│  │         (Control de Concurrencia)                      │     │
│  └───────────────────────────────────────────────────────┘     │
│                              │                                  │
│                              ▼                                  │
│  ┌───────────────────────────────────────────────────────┐     │
│  │              JobDispatcher                             │     │
│  │          (Gestor de Ejecución)                         │     │
│  └───────────────────────────────────────────────────────┘     │
│                              │                                  │
│                              ▼                                  │
│  ┌───────────────────────────────────────────────────────┐     │
│  │              job_executions                            │     │
│  │            (Tabla PostgreSQL)                          │     │
│  └───────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────┘
```

## Componentes Principales

### JobTracker - Control de Concurrencia

El `JobTracker` gestiona la concurrencia rastreando los trabajos en ejecución y determinando tamaños de lote para consultas.

| Componente | Tipo | Responsabilidad |
|------------|------|-----------------|
| running_jobs | AtomicUsize | Contador thread-safe de ejecuciones activas |
| notify | Notify | Notificación asíncrona para cambios de estado |
| next_batch_size() | método | Calcula cuántos trabajos consultar |
| dispatch_job() | método | Incrementa contador al iniciar trabajo |
| job_completed() | método | Decrementa contador y notifica |

```rust
pub fn next_batch_size(&self) -> Option<usize> {
    let n_running = self.running_jobs.load(Ordering::SeqCst);
    if n_running < self.min_jobs {
        Some(self.max_jobs - n_running)
    } else {
        None
    }
}
```

### JobDispatcher - Gestor de Ejecución

El `JobDispatcher` maneja el ciclo de vida completo de un trabajo: inicialización, ejecución, reintentos y finalización.

| Transición | Método | Operación DB |
|------------|--------|--------------|
| Pending → Running | execute_job() | SELECT + UPDATE state='running' |
| Running → Heartbeat | keep_job_alive() | UPDATE alive_at |
| Running → Complete | complete_job() | DELETE de job_executions |
| Running → Failed | fail_job() | UPDATE con reintento o DELETE |
| Running → Rescheduled | reschedule_job() | UPDATE execute_at |

### Ciclo de Vida del Trabajo

```
        ┌─────────┐
        │ Pending │
        └────┬────┘
             │ consultar
             ▼
        ┌─────────┐
        │ Running │◄──────────────┐
        └────┬────┘               │
             │                    │ heartbeat
             ├─────────┬──────────┘
             │         │
    éxito    │         │ error
             ▼         ▼
        ┌─────────┐ ┌─────────┐
        │Complete │ │ Failed  │
        └─────────┘ └────┬────┘
                         │ reintento
                         ▼
                    ┌─────────────┐
                    │ Rescheduled │
                    └─────────────┘
```

## Tipos de Trabajos

### Trabajos Impulsados por Eventos

Activados por eventos del outbox:

```rust
pub struct UserOnboardingJob {
    customers: Customers,
    outbox_consumer: OutboxConsumer,
}

impl Job for UserOnboardingJob {
    const NAME: &'static str = "user-onboarding";

    async fn run(&self, _: CurrentJob) -> Result<JobCompletion, JobError> {
        let events = self.outbox_consumer
            .poll::<CoreCustomerEvent>()
            .await?;

        for event in events {
            if let CoreCustomerEvent::KycCompleted { id, .. } = event.payload {
                self.customers.provision_user(id).await?;
            }
            self.outbox_consumer.ack(event.sequence).await?;
        }

        Ok(JobCompletion::Complete)
    }
}
```

### Trabajos Programados

Ejecución periódica tipo cron:

```rust
pub struct InterestAccrualJob {
    credit_facilities: CreditFacilities,
}

impl Job for InterestAccrualJob {
    const NAME: &'static str = "interest-accrual";

    async fn run(&self, _: CurrentJob) -> Result<JobCompletion, JobError> {
        self.credit_facilities.accrue_interest().await?;

        // Reprogramar para mañana
        Ok(JobCompletion::RescheduleAt(
            Utc::now() + Duration::days(1)
        ))
    }
}
```

### Trabajos de Sondeo

Auto-reprogramación para sincronización continua:

```rust
pub struct CollateralSyncJob {
    custody: Custody,
    price_service: PriceService,
}

impl Job for CollateralSyncJob {
    const NAME: &'static str = "collateral-sync";

    async fn run(&self, _: CurrentJob) -> Result<JobCompletion, JobError> {
        let wallets = self.custody.list_active_wallets().await?;

        for wallet in wallets {
            let balance = self.custody.sync_balance(&wallet).await?;
            let price = self.price_service.get_btc_price().await?;
            self.custody.update_collateral_value(&wallet, balance, price).await?;
        }

        // Reprogramar en 5 minutos
        Ok(JobCompletion::RescheduleIn(Duration::minutes(5)))
    }
}
```

## Esquema de Base de Datos

```sql
CREATE TABLE job_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_type TEXT NOT NULL,
    execute_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    state TEXT NOT NULL DEFAULT 'pending',
    attempt_index INT NOT NULL DEFAULT 0,
    alive_at TIMESTAMPTZ,
    payload JSONB,
    trace_context JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_job_executions_pending
    ON job_executions(execute_at)
    WHERE state = 'pending';

CREATE INDEX idx_job_executions_stale
    ON job_executions(alive_at)
    WHERE state = 'running';
```

## Lógica de Reintentos

### Configuración de RetrySettings

```rust
pub struct RetrySettings {
    pub max_attempts: u32,
    pub initial_interval: Duration,
    pub max_interval: Duration,
    pub multiplier: f64,
}

impl Default for RetrySettings {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_interval: Duration::seconds(5),
            max_interval: Duration::hours(1),
            multiplier: 2.0,
        }
    }
}
```

### Backoff Exponencial

```rust
fn calculate_next_attempt(&self, attempt: u32) -> Duration {
    let interval = self.initial_interval.as_secs_f64()
        * self.multiplier.powi(attempt as i32);

    Duration::seconds(
        interval.min(self.max_interval.as_secs_f64()) as i64
    )
}
```

## Mecanismo Keep-Alive

El sistema mantiene latidos para detectar trabajos que se han quedado colgados:

```rust
async fn keep_alive_loop(&self, job_id: Uuid, interval: Duration) {
    let mut ticker = tokio::time::interval(interval / 4);

    loop {
        ticker.tick().await;

        if let Err(e) = self.update_alive_at(job_id).await {
            tracing::warn!("Failed to update keep-alive: {}", e);
            break;
        }
    }
}
```

Los trabajos sin latido reciente se consideran fallidos y se reprograman.

## Observabilidad y Trazado

### Preservación del Contexto de Trazas

Los trabajos preservan el contexto de trazabilidad:

```rust
impl JobDispatcher {
    async fn execute_with_tracing(&self, job: CurrentJob) {
        // Restaurar contexto de traza del trabajo
        if let Some(trace_ctx) = &job.trace_context {
            let parent = trace_ctx.extract();
            let span = tracing::info_span!(
                "job.execute",
                job.type = job.job_type,
                job.id = %job.id
            );
            span.set_parent(parent);

            span.in_scope(|| {
                self.runner.run(job)
            }).await
        }
    }
}
```

### Puntos de Instrumentación

| Span | Propósito |
|------|-----------|
| `job.dispatch` | Ciclo de vida completo del trabajo |
| `job.execute` | Ejecución de la lógica del trabajo |
| `job.retry` | Intentos de reintento |
| `job.complete` | Finalización exitosa |
| `job.fail` | Fallo del trabajo |

## Integración con la Aplicación

### Registro de Trabajos

```rust
// lana/app/src/lib.rs
impl LanaApp {
    pub async fn register_jobs(&self, registry: &mut JobRegistry) {
        registry.register(InterestAccrualJob::new(
            self.credit_facilities.clone()
        ));

        registry.register(CollateralSyncJob::new(
            self.custody.clone(),
            self.price_service.clone()
        ));

        registry.register(UserOnboardingJob::new(
            self.customers.clone(),
            self.outbox_consumer.clone()
        ));
    }
}
```

### Configuración

```yaml
# config/jobs.yaml
jobs:
  min_concurrent: 2
  max_concurrent: 10
  poll_interval_ms: 1000
  keep_alive_interval_ms: 5000
  stale_threshold_ms: 30000

  retry:
    max_attempts: 5
    initial_interval_ms: 5000
    max_interval_ms: 3600000
    multiplier: 2.0
```

## Desarrollo Local

En desarrollo con Tilt, los trabajos se pueden monitorear en la UI de Tilt. Los logs de trabajos se envían al colector OTEL configurado.

```bash
# Ver trabajos pendientes
psql -c "SELECT * FROM job_executions WHERE state = 'pending'"

# Ver trabajos fallidos
psql -c "SELECT * FROM job_executions WHERE state = 'failed'"
```
