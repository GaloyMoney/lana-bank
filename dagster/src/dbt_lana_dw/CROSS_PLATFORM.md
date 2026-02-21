# Cross-Platform Data Warehouse Setup

This dbt project supports both BigQuery (production) and Postgres (local development).

## Environment Variables

### Target Selection
| Variable | Values | Default | Description |
|----------|--------|---------|-------------|
| `DW_TARGET` | `bigquery`, `postgres` | `bigquery` | Target data warehouse |

### Schema Configuration
| Variable | Description | Example |
|----------|-------------|---------|
| `DW_RAW_SCHEMA` | Schema for raw/source data loaded by dlt | `raw`, `john_raw` |
| `DW_DBT_SCHEMA` | Schema for dbt model outputs | `dbt`, `dbt_john` |

### BigQuery-specific
| Variable | Description |
|----------|-------------|
| `DBT_BIGQUERY_PROJECT` | GCP project ID |
| `DBT_BIGQUERY_CREDENTIALS_JSON` | Service account JSON (as string) |

### Postgres-specific
| Variable | Default | Description |
|----------|---------|-------------|
| `DW_PG_HOST` | `localhost` | Postgres host |
| `DW_PG_PORT` | `5432` | Postgres port |
| `DW_PG_DATABASE` | `lana_dw` | Database name |
| `DW_PG_USER` | `postgres` | Database user |
| `DW_PG_PASSWORD` | (empty) | Database password |

## Quick Start

### Local Development (Postgres)
```bash
# Start local Postgres
docker run -d --name lana_dw_pg \
  -e POSTGRES_DB=lana_dw \
  -e POSTGRES_PASSWORD=dev \
  -p 5432:5432 \
  postgres:15

# Set environment
export DW_TARGET=postgres
export DW_RAW_SCHEMA=raw
export DW_DBT_SCHEMA=dbt
export DW_PG_HOST=localhost
export DW_PG_PASSWORD=dev

# Create schemas
psql "postgresql://postgres:dev@localhost:5432/lana_dw" -c "
  CREATE SCHEMA IF NOT EXISTS raw;
  CREATE SCHEMA IF NOT EXISTS dbt;
"

# Run dbt
dbt run
```

### Production (BigQuery)
```bash
export DW_TARGET=bigquery
export DW_RAW_SCHEMA=prod_raw
export DW_DBT_SCHEMA=prod_dbt
export DBT_BIGQUERY_PROJECT=my-gcp-project
export DBT_BIGQUERY_CREDENTIALS_JSON='{"type":"service_account",...}'

dbt run
```

## Cross-Platform SQL Macros

The project uses macros in `macros/cross_db/` for SQL compatibility:

### `ident(name)` - Identifier quoting
```sql
-- Instead of: `column_name` (BigQuery) or "column_name" (Postgres)
{{ ident('column_name') }}
```

### `array_at(arr, idx)` - Array indexing
```sql
-- Instead of: arr[safe_offset(0)] (BigQuery) or arr[1] (Postgres)
{{ array_at('my_array', 0) }}
```

### `split_part_at(str, delimiter, idx)` - Split and get element
```sql
-- Instead of: split(name, ' ')[safe_offset(0)] (BigQuery)
{{ split_part_at('name', ' ', 0) }}
```

### `array_from_json_strings(json_col, path)` - JSON array to SQL array
```sql
-- Extract JSON array to SQL array
{{ array_from_json_strings('json_column') }}
{{ array_from_json_strings('json_column', 'nested.path') }}
```

## Known Limitations

1. **File reports**: Only work with BigQuery (require GCS integration)
2. **Some analytics queries**: May need manual review for complex aggregations
3. **JSON functions**: Basic JSON is supported; complex nested structures may need adjustment
