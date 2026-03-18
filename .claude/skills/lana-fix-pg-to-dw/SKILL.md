---
name: lana-fix-pg-to-dw
description: Diagnose and fix breaking changes in the PG-to-BigQuery data pipeline. Analyzes recent backend entity changes, the Dagster EL job, and dbt staging models to find and repair column/schema mismatches.
---

# Fix PG-to-Data-Warehouse Pipeline Breaks

Diagnose and fix breaking changes introduced by backend development that affect the PG → BigQuery → dbt staging pipeline.

## Context

$ARGUMENTS

## Data Pipeline Overview

The data flows through three layers:

1. **Backend entities** (Rust, event sourcing) → PG tables (events + rollup tables)
2. **Dagster EL job** → loads PG tables to BigQuery (`dagster/src/assets/lana.py`)
3. **dbt staging models** → transform raw BQ tables into analytics-ready views (`dagster/src/dbt_lana_dw/models/staging/`)

### Key Files

| Layer | Files |
|-------|-------|
| Entities | `core/*/src/*/entity.rs` — event definitions with field names |
| Rollup migrations | `lana/app/migrations/*_create_core_*_events_rollup.sql` — trigger functions that flatten events into rollup columns |
| EL table list | `dagster/src/assets/lana.py` — `LANA_EL_TABLE_NAMES` array (which PG tables get loaded) |
| Type mapping | `dagster/src/utils/pg_to_bq_type_mapping.py` — PG→BQ column type conversion |
| dbt sources | `dagster/src/dbt_lana_dw/models/sources.yml` — BQ source table declarations |
| dbt staging models | `dagster/src/dbt_lana_dw/models/staging/stg_*.sql` and `staging/rollups/stg_*.sql` |
| dbt schema tests | `dagster/src/dbt_lana_dw/models/staging/stg_*.yml` |

## Step 1: Identify the Breaking Change

**Always start by running the column consistency checker:**

```bash
python dev/check-dbt-columns.py
```

This script (`dev/check-dbt-columns.py`) uses `sqlglot` to parse both PG migration DDL and dbt staging SQL, and reports three kinds of issues:
- **SELECT * usage** — staging models must list columns explicitly
- **Missing source tables** — dbt references a source with no `CREATE TABLE` in migrations
- **Column mismatches** — dbt references columns that don't exist in the migration-defined schema

Use `--verbose` to see all parsed table schemas and their columns. The output pinpoints exactly which model, table, and columns are mismatched — use this to drive the fix.

Then gather additional context as needed:

### 1a. If the user provided error output or context
Parse the error to identify which dbt model failed and which column/table is missing or mistyped.

### 1b. Check recent backend changes
Look at recent commits that touch entities, migrations, or rollup tables:

```bash
git log --oneline -20 -- 'core/*/src/*/entity.rs' 'lana/app/migrations/'
```

For each relevant commit, check what changed:
```bash
git diff <commit>~1 <commit> -- 'core/*/src/*/entity.rs' 'lana/app/migrations/'
```

Focus on:
- **New/renamed/removed fields** in entity event variants
- **New/modified rollup migrations** that change trigger functions or table columns
- **New entity tables** that may need to be added to the EL pipeline

### 1c. Compare rollup table columns against dbt models
For the affected entity, trace the full chain:

1. Read the rollup migration to see what columns the PG table has
2. Check if the table is in `LANA_EL_TABLE_NAMES` in `dagster/src/assets/lana.py`
3. Check if the table is declared in `dagster/src/dbt_lana_dw/models/sources.yml`
4. Read the corresponding `stg_*.sql` model to see what columns it references

## Step 2: Classify the Break — Mechanical vs Semantic

This is the critical decision point. Classify each issue as **mechanical** or **semantic** before touching any code.

### Mechanical changes (safe to fix automatically)
These are straightforward mappings where the intent is obvious:

| Type | Example | Action |
|------|---------|--------|
| **Column renamed** | `collateral_ratio` → `collateralization_ratio` | Rename in dbt model |
| **Column added** (passthrough) | New field appears in rollup, no logic change | Add column to staging model |
| **Column removed** (unused) | Field dropped from rollup, dbt references it but nothing downstream uses it | Remove from staging model |
| **New table** (mechanical) | New entity rollup migration added | Wire up EL + source + staging model |
| **JSON path renamed** | `json_value(x, '$.old_name')` → `json_value(x, '$.new_name')` | Update path |

### Semantic changes (DO NOT auto-fix — report only)
These involve domain meaning changes where guessing the right dbt transformation is dangerous:

| Signal | Example |
|--------|---------|
| **Entity split or merged** | One rollup table replaced by two, or two consolidated into one |
| **Column semantics changed** | `amount` was in sats, now in BTC; `status` enum values restructured |
| **Business logic moved** | Field computed differently (was a direct column, now derived from multiple fields) |
| **Downstream models depend on removed columns** | Intermediate/mart models use a column that no longer exists with the same meaning |
| **Multiple columns changed together** | Suggests a domain concept was reworked, not just renamed |
| **New JSONB structure** | Complex nested JSON replaced with a different shape — not a simple path rename |

**When semantic changes are detected, STOP and report instead of fixing.** Present:
1. What changed in the backend (the commit, the entity diff, the migration diff)
2. Which dbt models and downstream consumers are affected
3. What the old vs new schema looks like
4. Specific questions the data team needs to answer to define the correct transformation

Do not guess how to map changed domain concepts to dbt SQL.

## Step 3: Apply the Fix (Mechanical Changes Only)

Only proceed with fixes for issues classified as **mechanical** in Step 2. For semantic changes, skip to Step 4 and include a report for the user.

Work bottom-up from dbt to EL to keep changes minimal.

### Fixing dbt staging models
- Edit the `stg_*.sql` file to match the current rollup table schema
- Update column references, JSON paths, aliases
- If columns were added, decide whether to expose them (check if downstream models need them)
- Update corresponding `.yml` test files if they exist

### Fixing EL pipeline
- If a new table needs loading: add it to `LANA_EL_TABLE_NAMES` in `dagster/src/assets/lana.py`
- If a table was removed: remove it from `LANA_EL_TABLE_NAMES`
- The EL job infers schema dynamically from PG, so column changes within existing tables are handled automatically

### Fixing dbt sources
- If a new table was added to EL: add a corresponding entry in `dagster/src/dbt_lana_dw/models/sources.yml` under the `lana` source
- If a table was removed: remove it from `sources.yml`

### Creating a new staging model
When a completely new rollup table is introduced:

1. Add the table to `LANA_EL_TABLE_NAMES` in `dagster/src/assets/lana.py`
2. Add the source in `dagster/src/dbt_lana_dw/models/sources.yml`
3. Create a new staging model following the existing pattern:

```sql
with raw as (
    select * from {{ source("lana", "<table_name>") }}
),
ordered as (
    select
        *,
        row_number() over (
            partition by id order by _dlt_load_id desc
        ) as order_received_desc
    from raw
)
select
    id as <entity>_id,
    -- map columns from the rollup migration
    version,
    created_at,
    modified_at,
    timestamp_micros(
        cast(cast(_dlt_load_id as decimal) * 1e6 as int64)
    ) as loaded_to_dw_at
from ordered
where order_received_desc = 1
```

## Step 4: Validate

1. **Re-run the column checker** to confirm the fix resolves all issues:
   ```bash
   python dev/check-dbt-columns.py
   ```
   The script must exit 0 with "All dbt staging models reference valid source columns."

2. **Check downstream models**: Search for references to the modified staging model in `dagster/src/dbt_lana_dw/models/` to ensure downstream consumers aren't broken:
   ```bash
   grep -r "stg_<model_name>" dagster/src/dbt_lana_dw/models/
   ```

## Step 5: Commit

Create a local commit with a conventional commit message, e.g.:
- `fix(dagster): update stg_core_party_events_rollup for new email column`
- `feat(dagster): add core_withdrawal_events_rollup to EL pipeline and staging`

## Guidelines

- **Minimal changes**: Only fix what's broken. Don't refactor unrelated models.
- **Follow existing patterns**: Match the style of neighboring staging models (dedup pattern, `loaded_to_dw_at` column, aliasing `id` to `<entity>_id`).
- **Rollup migrations are the source of truth**: The rollup table schema (defined in migration trigger functions) determines what columns exist in PG. Always read the migration to understand the actual schema.
- **Don't modify backend code**: This skill fixes the data pipeline to match backend changes, not the other way around.
- **JSON fields**: Rollup tables often store complex nested data as JSONB columns. dbt models parse these with `json_value()`. When event field structures change, the JSON paths in dbt must be updated to match.
