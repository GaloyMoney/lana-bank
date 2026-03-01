---
id: data-pipelines
title: Data Pipelines
sidebar_position: 12
---

# Data Pipelines

This document describes the data pipeline architecture using Meltano, dbt, and BigQuery.

```mermaid
graph TD
    subgraph Sources["Data Sources"]
        PG["PostgreSQL<br/>(Core Operational DB)"]
        BITFINEX["Bitfinex API<br/>(BTC/USD Prices)"]
        SUMSUB["Sumsub API<br/>(KYC Data)"]
    end

    subgraph Orchestration
        AIRFLOW["Airflow<br/>(Optional)"]
        SCHEDULES["Meltano Schedules"]
        JOBS["Meltano Jobs"]
        SCHEDULES --> JOBS
        AIRFLOW -.->|"optional orchestration"| SCHEDULES
    end

    subgraph Extract["Meltano Extracting"]
        TAP_PG["tap-postgres<br/>(MeltanoLabs variant)"]
        TAP_BIT["tap-bitfinex<br/>(Custom implementation)"]
        TAP_SUM["tap-sumsubapi<br/>(Custom implementation)"]
    end

    subgraph Load["Meltano Loading"]
        TARGET_BQ["target-bigquery<br/>(Adswerve variant)"]
    end

    subgraph Transform["dbt Transformations"]
        DBT_BQ["dbt-bigquery<br/>(dbt-labs 1.8.1)"]
        MODELS["Transformation Models"]
        TESTS["Data Quality Tests"]
        DBT_BQ --> MODELS
        DBT_BQ --> TESTS
    end

    subgraph Warehouse["BigQuery Datasets"]
        RAW["Raw Dataset<br/>(LANA_dataset)"]
        MART["dbt Dataset<br/>(dbt_LANA)"]
    end

    subgraph Reports
        GEN["generate-es-reports<br/>(Custom utility)"]
        FIN["Financial Reports"]
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

## Overview

The data pipeline provides:

- Data extraction from operational systems
- Transformation for analytics
- Data warehouse for reporting
- Orchestration with Dagster

## Architecture

```mermaid
graph TD
    SRC["Source Systems<br/>(PostgreSQL, Cala Ledger, External APIs)"] --> MELT["Meltano<br/>(Extract and Load)"]
    MELT --> BQ["BigQuery<br/>(Data Warehouse)"]
    BQ --> DBT["dbt<br/>(Transformations)"]
    DBT --> DAG["Dagster<br/>(Orchestration)"]
```

## Meltano Configuration

### Extractors

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

### Loaders

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

## dbt Transformations

### Model Structure

```
dagster/dbt/
├── models/
│   ├── staging/          # Clean raw data
│   │   ├── stg_customers.sql
│   │   ├── stg_facilities.sql
│   │   └── stg_transactions.sql
│   ├── intermediate/     # Business logic
│   │   ├── int_customer_metrics.sql
│   │   └── int_facility_performance.sql
│   └── marts/            # Final reporting
│       ├── fct_disbursals.sql
│       ├── fct_payments.sql
│       └── dim_customers.sql
└── dbt_project.yml
```

### Example Model

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

### Intermediate Model

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

## Dagster Orchestration

### Asset Definition

```python
# dagster/assets.py
from dagster import asset
from dagster_dbt import dbt_assets

@dbt_assets(manifest_path=DBT_MANIFEST_PATH)
def dbt_models(context):
    yield from build_dbt_assets(context)

@asset(deps=[dbt_models])
def customer_report():
    """Generate daily customer report."""
    query = """
    SELECT * FROM `project.dataset.dim_customers`
    """
    df = bigquery_client.query(query).to_dataframe()
    return df
```

### Schedule

```python
from dagster import ScheduleDefinition

daily_pipeline = ScheduleDefinition(
    job=etl_job,
    cron_schedule="0 2 * * *",  # 2 AM daily
)
```

## Data Quality

### dbt Tests

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
              values: ['ACTIVE', 'INACTIVE', 'SUSPENDED']
```

### Custom Tests

```sql
-- tests/assert_positive_balances.sql
select
    facility_id,
    outstanding_balance
from {{ ref('fct_facilities') }}
where outstanding_balance < 0
```

## Reporting Views

### Financial Summary

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

## Access Control

### BigQuery Permissions

| Role | Access Level |
|------|--------------|
| Data Engineer | Full access |
| Analyst | Read marts only |
| Reporting | Read specific views |

