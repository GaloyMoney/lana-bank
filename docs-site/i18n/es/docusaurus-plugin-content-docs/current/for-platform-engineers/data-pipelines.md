---
id: data-pipelines
title: Canalización de Datos
sidebar_position: 4
---

# Canalización de Datos y Analítica

Este documento describe la infraestructura de canalización de datos y analítica utilizada para extraer datos operativos del sistema Lana Bank, transformarlos con fines analíticos y cargarlos en BigQuery para reporting e inteligencia de negocio.

![Arquitectura del Pipeline de Datos](/img/architecture/data-pipeline-1.png)

## Arquitectura de la Canalización

La canalización de datos sigue un patrón ELT (Extract, Load, Transform) usando Meltano como marco de orquestación. Los datos fluyen desde múltiples sistemas origen hacia BigQuery, donde dbt aplica transformaciones para crear modelos analíticos.

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Fuentes de Datos                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                 │
│  │ PostgreSQL  │  │ Bitfinex    │  │   Sumsub    │                 │
│  │  (Core DB)  │  │    API      │  │    API      │                 │
│  └─────────────┘  └─────────────┘  └─────────────┘                 │
└─────────────────────────────────────────────────────────────────────┘
            │                │                │
            ▼                ▼                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      Extractores (Meltano)                          │
│  ┌─────────────┐  ┌─────────────────┐  ┌──────────────┐            │
│  │tap-postgres │  │ tap-bitfinexapi │  │tap-sumsubapi │            │
│  └─────────────┘  └─────────────────┘  └──────────────┘            │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      Cargador (Meltano)                             │
│              ┌────────────────────────────┐                         │
│              │    target-bigquery         │                         │
│              └────────────────────────────┘                         │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      Transformaciones (dbt)                         │
│              ┌────────────────────────────┐                         │
│              │   Modelos Analíticos       │                         │
│              └────────────────────────────┘                         │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         BigQuery                                    │
│              ┌────────────────────────────┐                         │
│              │   Almacén de Datos         │                         │
│              └────────────────────────────┘                         │
└─────────────────────────────────────────────────────────────────────┘
```

## Fuentes de Datos

### PostgreSQL - Datos Operativos

La fuente de datos principal es la base de datos PostgreSQL que contiene todos los datos operativos del sistema. El extractor `tap-postgres` extrae datos de tablas de eventos, vistas de rollup y tablas del libro mayor.

#### Tablas Extraídas

| Categoría | Tablas | Propósito |
|-----------|--------|-----------|
| Libro mayor | `cala_balance_history`, `cala_accounts`, `cala_account_sets` | Datos de contabilidad de partida doble |
| Créditos | `core_credit_facility_events`, `core_credit_facility_proposal_events` | Ciclo de vida de líneas de crédito |
| Depósitos | `core_deposit_accounts`, `core_deposit_events`, `core_withdrawal_events` | Operaciones de cuentas de depósito |
| Pagos | `core_obligation_events`, `core_payment_events`, `core_payment_allocation_events` | Procesamiento y asignación de pagos |
| Desembolsos | `core_disbursal_events` | Desembolsos de líneas de crédito |
| Intereses | `core_interest_accrual_cycle_events` | Ciclos de cálculo de intereses |
| Colateral | `core_collateral_events` | Seguimiento de colateral en Bitcoin |
| Clientes | `core_customer_events` | Ciclo de vida de clientes |
| Gobernanza | `core_approval_process_events_rollup`, `core_committee_events_rollup` | Flujos de aprobación |
| Control de Acceso | `core_user_events_rollup`, `core_role_events_rollup` | Sistema RBAC |
| Contabilidad | `core_chart_events`, `core_manual_transaction_events_rollup` | Plan de cuentas |

### API de Bitfinex - Datos de Precios

El `tap-bitfinexapi` es un extractor personalizado que consulta la API de Bitfinex para datos del mercado BTC/USD. Estos datos son críticos para cálculos de colateralización y gestión de riesgo.

**Datos extraídos:**
- `bitfinex_ticker` - Información actual del ticker (bid, ask, último precio)
- `bitfinex_trades` - Historial reciente de operaciones
- `bitfinex_order_book` - Profundidad actual del libro de órdenes

El extractor se ejecuta cada minuto para mantener información de precio actualizada.

### API de Sumsub - Datos KYC

El `tap-sumsubapi` es un extractor personalizado que recupera datos de verificación KYC de Sumsub. A diferencia de los otros extractores, este tap es stateful y consulta PostgreSQL para determinar qué registros de clientes deben sincronizarse.

**Flujo del extractor:**
1. Consulta PostgreSQL para recuperar IDs de clientes actualizados
2. Obtiene datos del solicitante de la API de Sumsub
3. Descarga imágenes de documentos y las codifica en base64
4. Emite registros estructurados

## Configuración de Meltano

### Estructura de la Canalización

La canalización se configura en `meltano.yml`:

```yaml
plugins:
  extractors:
    - name: tap-postgres
      config:
        host: ${PG_HOST}
        port: ${PG_PORT}
        user: ${PG_USER}
        password: ${PG_PASSWORD}
        database: ${PG_DATABASE}

    - name: tap-bitfinexapi
      namespace: tap_bitfinexapi

    - name: tap-sumsubapi
      namespace: tap_sumsubapi

  loaders:
    - name: target-bigquery
      config:
        project_id: ${BIGQUERY_PROJECT}
        dataset_id: ${BIGQUERY_DATASET}
```

### Schedules y Jobs

```yaml
schedules:
  - name: postgres-to-bigquery
    interval: "@hourly"
    job: el-postgres

  - name: bitfinex-to-bigquery
    interval: "*/1 * * * *"  # Cada minuto
    job: el-bitfinex

  - name: sumsub-to-bigquery
    interval: "@daily"
    job: el-sumsub

jobs:
  - name: el-postgres
    tasks:
      - tap-postgres target-bigquery

  - name: el-bitfinex
    tasks:
      - tap-bitfinexapi target-bigquery

  - name: el-sumsub
    tasks:
      - tap-sumsubapi target-bigquery
```

## Transformaciones con dbt

### Configuración de Sources

Las fuentes de dbt se definen en `models/sources.yml`:

```yaml
version: 2

sources:
  - name: lana_raw
    database: "{{ env_var('BIGQUERY_PROJECT') }}"
    schema: "{{ env_var('BIGQUERY_DATASET') }}"
    tables:
      - name: core_credit_facility_events
        freshness:
          warn_after: {count: 2, period: hour}
          error_after: {count: 6, period: hour}
      - name: core_deposit_events
      - name: cala_balance_history
```

### Monitoreo de Frescura

dbt monitorea la frescura de los datos y alerta cuando los datos están desactualizados:

```bash
dbt source freshness
```

## Configuración de Entornos

### Desarrollo

```yaml
environments:
  - name: dev
    config:
      plugins:
        loaders:
          - name: target-bigquery
            config:
              dataset_id: lana_dev
```

### Producción

```yaml
environments:
  - name: prod
    config:
      plugins:
        loaders:
          - name: target-bigquery
            config:
              dataset_id: lana_prod
```

## Herramientas Adicionales

### sqlfluff - Linting de SQL

Para mantener la calidad del código SQL:

```bash
sqlfluff lint models/
sqlfluff fix models/
```

### Airflow - Orquestación Opcional

Para orquestación avanzada, Airflow puede usarse como alternativa a los schedules de Meltano:

```python
from airflow import DAG
from airflow.operators.bash import BashOperator

with DAG('lana_etl', schedule_interval='@hourly') as dag:
    extract = BashOperator(
        task_id='extract',
        bash_command='meltano run tap-postgres target-bigquery'
    )
```

## Ejecución de la Canalización

### Comandos Básicos

```bash
# Ejecutar extracción completa
meltano run tap-postgres target-bigquery

# Ejecutar con selección de tablas
meltano run tap-postgres target-bigquery --select core_credit_facility_events

# Ejecutar transformaciones dbt
meltano invoke dbt:run

# Verificar frescura de datos
meltano invoke dbt:source freshness
```

### Desarrollo Local

Para desarrollo local con Dagster:

```bash
DAGSTER=true make start-deps
```

Esto inicia la interfaz de Dagster para visualizar y ejecutar la canalización.
