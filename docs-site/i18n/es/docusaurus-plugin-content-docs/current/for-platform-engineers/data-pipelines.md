---
id: data-pipelines
title: Pipelines de Datos
sidebar_position: 12
---

# Pipelines de Datos

Este documento describe la arquitectura del pipeline de datos utilizando Meltano, dbt y BigQuery.

```mermaid
graph TD
    subgraph Sources["Fuentes de Datos"]
        PG["PostgreSQL<br/>(Base de Datos Operacional Principal)"]
        BITFINEX["API de Bitfinex<br/>(Precios BTC/USD)"]
        SUMSUB["API de Sumsub<br/>(Datos KYC)"]
    end

    subgraph Orchestration
        AIRFLOW["Airflow<br/>(Opcional)"]
        SCHEDULES["Programaciones de Meltano"]
        JOBS["Trabajos de Meltano"]
        SCHEDULES --> JOBS
        AIRFLOW -.->|"orquestación opcional"| SCHEDULES
    end

    subgraph Extract["Extracción con Meltano"]
        TAP_PG["tap-postgres<br/>(Variante MeltanoLabs)"]
        TAP_BIT["tap-bitfinex<br/>(Implementación personalizada)"]
        TAP_SUM["tap-sumsubapi<br/>(Implementación personalizada)"]
    end

    subgraph Load["Carga con Meltano"]
        TARGET_BQ["target-bigquery<br/>(Variante Adswerve)"]
    end

    subgraph Transform["Transformaciones dbt"]
        DBT_BQ["dbt-bigquery<br/>(dbt-labs 1.8.1)"]
        MODELS["Modelos de Transformación"]
        TESTS["Pruebas de Calidad de Datos"]
        DBT_BQ --> MODELS
        DBT_BQ --> TESTS
    end

    subgraph Warehouse["Conjuntos de Datos de BigQuery"]
        RAW["Conjunto de Datos Sin Procesar<br/>(LANA_dataset)"]
        MART["Conjunto de Datos dbt<br/>(dbt_LANA)"]
    end

    subgraph Reports
        GEN["generate-es-reports<br/>(Utilidad personalizada)"]
        FIN["Informes Financieros"]
    end

    PG --> TAP_PG
    BITFINEX --> TAP_BIT
    SUMSUB --> TAP_SUM
    TAP_PG --> TARGET_BQ
    TAP_BIT --> TARGET_BQ
    TAP_SUM --> TARGET_BQ
    TARGET_BQ --> RAW
    RAW --> DBT_BQ
    DBT_BQ --> MART
    MART --> GEN
    GEN --> FIN
```

## Descripción General

El pipeline de datos proporciona:

- Extracción de datos desde sistemas operacionales
- Transformación para análisis
- Almacén de datos para informes
- Orquestación con Dagster

## Arquitectura

```mermaid
graph TD
    SRC["Sistemas Fuente<br/>(PostgreSQL, Cala Ledger, APIs Externas)"] --> MELT["Meltano<br/>(Extraer y Cargar)"]
    MELT --> BQ["BigQuery<br/>(Almacén de Datos)"]
    BQ --> DBT["dbt<br/>(Transformaciones)"]
    DBT --> DAG["Dagster<br/>(Orquestación)"]
```

## Configuración de Meltano

### Extractores

```yaml

# meltano.yml

plugins:
  extractors:
    - name: tap-postgres
      variant: meltanolabs
      config:
        host: ${POSTGRES_HOST}
        port: 5432
        user: ${POSTGRES_USER}
        password: ${POSTGRES_PASSWORD}
        database: lana
```

### Cargadores

```yaml
plugins:
  loaders:
    - name: target-bigquery
      variant: adswerve
      config:
        project_id: ${GCP_PROJECT_ID}
        dataset_id: lana_raw
        location: US
```

## Transformaciones dbt

### Estructura del Modelo

```
dagster/dbt/
├── models/
│   ├── staging/          # Limpiar datos sin procesar
│   │   ├── stg_customers.sql
│   │   ├── stg_facilities.sql
│   │   └── stg_transactions.sql
│   ├── intermediate/     # Lógica de negocio
│   │   ├── int_customer_metrics.sql
│   │   └── int_facility_performance.sql
│   └── marts/            # Informes finales
│       ├── fct_disbursals.sql
│       ├── fct_payments.sql
│       └── dim_customers.sql
└── dbt_project.yml
```

### Modelo de Ejemplo

```sql
-- models/staging/stg_customers.sql
{{ config(materialized='view') }}

select
    id as customer_id,
    email,
    status,
    customer_type,
    created_at,
    updated_at
from {{ source('lana_raw', 'customers') }}
where _sdc_deleted_at is null
```

### Modelo Intermedio

```sql
-- models/intermediate/int_customer_metrics.sql
{{ config(materialized='table') }}

select
    c.customer_id,
    count(distinct f.facility_id) as total_facilities,
    sum(f.principal_amount) as total_principal,
    sum(f.outstanding_balance) as total_outstanding,
    max(f.created_at) as last_facility_date
from {{ ref('stg_customers') }} c
left join {{ ref('stg_facilities') }} f
    on c.customer_id = f.customer_id
group by 1
```

## Orquestación con Dagster

### Definición de Asset

```python

# dagster/assets.py

from dagster import asset
from dagster_dbt import dbt_assets

@dbt_assets(manifest_path=DBT_MANIFEST_PATH)
def dbt_models(context):
    yield from build_dbt_assets(context)

@asset(deps=[dbt_models])
def customer_report():
    """Generar informe diario de clientes."""
    query = """
    SELECT * FROM `project.dataset.dim_customers`
    """
    df = bigquery_client.query(query).to_dataframe()
    return df
```

### Programación

```python
from dagster import ScheduleDefinition

daily_pipeline = ScheduleDefinition(
    job=etl_job,
    cron_schedule="0 2 * * *",  # 2 AM daily
)
```

## Calidad de Datos

### Pruebas de dbt

```yaml

# models/schema.yml

version: 2

models:
  - name: stg_customers
    columns:
      - name: customer_id
        tests:
          - unique
          - not_null
      - name: email
        tests:
          - not_null
          - unique
      - name: status
        tests:
          - accepted_values:
              values: ['ACTIVE', 'INACTIVE', 'ESCHEATABLE']
```

### Pruebas Personalizadas

```sql
-- tests/assert_positive_balances.sql
select
    facility_id,
    outstanding_balance
from {{ ref('fct_facilities') }}
where outstanding_balance < 0
```

## Vistas de Informes

### Resumen Financiero

```sql
-- models/marts/financial_summary.sql
select
    date_trunc('month', transaction_date) as month,
    sum(case when type = 'DEPOSIT' then amount end) as deposits,
    sum(case when type = 'WITHDRAWAL' then amount end) as withdrawals,
    sum(case when type = 'DISBURSAL' then amount end) as disbursals,
    sum(case when type = 'PAYMENT' then amount end) as payments
from {{ ref('fct_transactions') }}
group by 1
order by 1
```

## Control de Acceso

### Permisos de BigQuery

| Rol | Nivel de Acceso |
|------|--------------|
| Ingeniero de Datos | Acceso completo |
| Analista | Solo lectura de marts |
| Reportes | Lectura de vistas específicas |
