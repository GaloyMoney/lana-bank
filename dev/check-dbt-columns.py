#!/usr/bin/env python3
"""
Validates that dbt staging models only reference columns that exist in upstream
PostgreSQL source tables (rollup tables and other core tables).

Parses migration SQL to extract CREATE TABLE column definitions, then uses
sqlglot to parse dbt staging SQL and extract column references from source CTEs.

Usage:
    python dev/check-dbt-columns.py
    python dev/check-dbt-columns.py --verbose
"""

import argparse
import re
import sys
from pathlib import Path

import sqlglot
from sqlglot import exp

# dlt always adds these columns to every table it loads
DLT_COLUMNS = {"_dlt_load_id", "_dlt_id"}

# Tables whose schema is managed externally (CALA, inbox, bitfinex, sumsub).
# We skip validation for these since they don't come from entity-rollups.
SKIP_TABLES = {
    "cala_account_sets",
    "cala_accounts",
    "cala_account_set_member_accounts",
    "cala_account_set_member_account_sets",
    "cala_balance_history",
    "cala_cumulative_effective_balances",
    "inbox_events",
    "bitfinex_ticker_dlt",
    "bitfinex_trades_dlt",
    "bitfinex_order_book_dlt",
    "bitfinex_order_book_dlt__orders",
    "sumsub_applicants_dlt",
}


# ---------------------------------------------------------------------------
# Migration parsing (CREATE TABLE -> column names)
# ---------------------------------------------------------------------------


def parse_create_table(sql: str) -> dict[str, set[str]]:
    """Extract column names from CREATE TABLE statements in migration SQL."""
    tables = {}
    pattern = re.compile(
        r"CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?(\w+)\s*\((.*?)\);",
        re.DOTALL | re.IGNORECASE,
    )
    for match in pattern.finditer(sql):
        tbl_name = match.group(1)
        body = match.group(2)
        columns = set()
        for line in body.split("\n"):
            line = line.strip()
            if not line or line.startswith("--") or line.startswith("/*"):
                continue
            if any(
                line.upper().startswith(kw)
                for kw in ("PRIMARY", "UNIQUE", "FOREIGN", "CHECK", "CONSTRAINT", ")")
            ):
                continue
            col_match = re.match(r"(\w+)\s+", line)
            if col_match:
                columns.add(col_match.group(1).lower())
        if columns:
            tables[tbl_name] = columns
    return tables


def parse_create_table_from_comment(sql: str) -> dict[str, set[str]]:
    """Extract columns from CREATE TABLE inside SQL block comments.

    Update migrations embed the current table structure as:
        /* CREATE TABLE ... */
    """
    tables = {}
    for comment_match in re.finditer(r"/\*(.*?)\*/", sql, re.DOTALL):
        tables.update(parse_create_table(comment_match.group(1)))
    return tables


def load_migration_schemas(migrations_dir: Path) -> dict[str, set[str]]:
    """Load column schemas from all migration files, latest wins."""
    schemas: dict[str, set[str]] = {}

    for migration_file in sorted(migrations_dir.glob("*.sql")):
        sql = migration_file.read_text()
        filename = migration_file.name

        if "_create_" in filename and "events_rollup" in filename:
            schemas.update(parse_create_table(sql))
        elif "_update_" in filename and "events_rollup" in filename:
            tables = parse_create_table_from_comment(sql)
            if tables:
                schemas.update(tables)
        elif filename == "20240517074612_core_setup.sql":
            schemas.update(parse_create_table(sql))

    return schemas


# ---------------------------------------------------------------------------
# dbt SQL parsing (sqlglot-based column extraction)
# ---------------------------------------------------------------------------


def preprocess_dbt_sql(sql: str) -> tuple[str, dict[str, str]]:
    """Replace dbt Jinja expressions with parseable SQL and track source tables.

    Returns (clean_sql, {cte_table_alias: source_table_name}).
    """
    source_map: dict[str, str] = {}

    # Replace {{ source("lana", "TABLE") }} with a placeholder table name
    def replace_source(m: re.Match) -> str:
        table_name = m.group(1)
        placeholder = f"__source__{table_name}"
        source_map[placeholder] = table_name
        return placeholder

    clean = re.sub(
        r"\{\{\s*source\(\s*[\"']lana[\"']\s*,\s*[\"'](\w+)[\"']\s*\)\s*\}\}",
        replace_source,
        sql,
    )

    # Remove remaining Jinja blocks (config, ref, etc.)
    clean = re.sub(r"\{\{.*?\}\}", "", clean, flags=re.DOTALL)
    clean = re.sub(r"\{%.*?%\}", "", clean, flags=re.DOTALL)

    return clean, source_map


def find_columns_from_source(node: exp.Expression) -> set[str]:
    """Walk a sqlglot AST node and collect all Column references."""
    columns = set()
    for col in node.find_all(exp.Column):
        columns.add(col.name.lower())
    return columns


def extract_source_columns_from_dbt(sql: str) -> list[tuple[str, list[str]]]:
    """Parse a dbt staging SQL file and extract (source_table, [columns]) pairs.

    Uses sqlglot to properly parse the SQL AST, handling CASE, nested functions, etc.
    """
    clean_sql, source_map = preprocess_dbt_sql(sql)

    if not source_map:
        return []

    try:
        parsed = sqlglot.parse_one(clean_sql, dialect="bigquery")
    except sqlglot.errors.ParseError:
        return []

    results = []

    # Strategy: find CTEs or subqueries that SELECT FROM a source placeholder,
    # then collect all Column nodes within that SELECT.
    for select in parsed.find_all(exp.Select):
        from_clause = select.find(exp.From)
        if not from_clause:
            continue

        table_node = from_clause.find(exp.Table)
        if not table_node:
            continue

        table_ref = table_node.name
        if table_ref not in source_map:
            continue

        source_table = source_map[table_ref]

        # Collect column names from this SELECT's expressions
        columns = []
        for select_expr in select.expressions:
            # Find all column references in this expression
            for col in select_expr.find_all(exp.Column):
                col_name = col.name.lower()
                if col_name not in columns:
                    columns.append(col_name)

            # Handle Star (SELECT *)
            if isinstance(select_expr, exp.Star):
                columns = []
                break

        if columns:
            results.append((source_table, columns))

    return results


# ---------------------------------------------------------------------------
# Validation and reporting
# ---------------------------------------------------------------------------


class ColumnMismatch:
    def __init__(
        self,
        dbt_file: str,
        source_table: str,
        missing_columns: list[str],
        available_columns: set[str],
    ):
        self.dbt_file = dbt_file
        self.source_table = source_table
        self.missing_columns = missing_columns
        self.available_columns = available_columns


def validate(
    migrations_dir: Path, dbt_staging_dir: Path, verbose: bool = False
) -> list[ColumnMismatch]:
    """Compare dbt column refs against migration schemas."""

    schemas = load_migration_schemas(migrations_dir)

    if verbose:
        print(f"Loaded schemas for {len(schemas)} tables from migrations")
        for tbl, cols in sorted(schemas.items()):
            print(f"  {tbl}: {sorted(cols)}")
        print()

    # dlt adds metadata columns to every table
    for tbl in schemas:
        schemas[tbl] |= DLT_COLUMNS

    mismatches = []

    for dbt_file in sorted(dbt_staging_dir.rglob("*.sql")):
        sql = dbt_file.read_text()
        source_refs = extract_source_columns_from_dbt(sql)

        for source_table, referenced_columns in source_refs:
            if source_table in SKIP_TABLES:
                continue

            if source_table not in schemas:
                if verbose:
                    print(
                        f"  WARN: {dbt_file.name} references source '{source_table}' "
                        f"but no CREATE TABLE found in migrations (may be external)"
                    )
                continue

            available = schemas[source_table]
            missing = [c for c in referenced_columns if c not in available]

            if missing:
                mismatches.append(
                    ColumnMismatch(
                        dbt_file=str(dbt_file.relative_to(dbt_staging_dir.parents[2])),
                        source_table=source_table,
                        missing_columns=missing,
                        available_columns=available,
                    )
                )
            elif verbose:
                print(f"  OK: {dbt_file.name} -> {source_table}")

    return mismatches


def find_downstream_models(dbt_models_dir: Path, staging_model_name: str) -> list[str]:
    """Find dbt models that reference a given staging model via ref()."""
    downstream = []
    for sql_file in dbt_models_dir.rglob("*.sql"):
        content = sql_file.read_text()
        if f"ref('{staging_model_name}')" in content or f'ref("{staging_model_name}")' in content:
            downstream.append(sql_file.stem)
    return downstream


def main():
    parser = argparse.ArgumentParser(
        description="Validate dbt staging models against PostgreSQL migration schemas"
    )
    parser.add_argument(
        "--migrations-dir",
        default="lana/app/migrations",
        help="Path to SQL migration files",
    )
    parser.add_argument(
        "--dbt-dir",
        default="dagster/src/dbt_lana_dw/models/staging",
        help="Path to dbt staging models directory",
    )
    parser.add_argument(
        "--dbt-models-dir",
        default="dagster/src/dbt_lana_dw/models",
        help="Path to all dbt models (for downstream impact analysis)",
    )
    parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parent.parent
    migrations_dir = repo_root / args.migrations_dir
    dbt_staging_dir = repo_root / args.dbt_dir
    dbt_models_dir = repo_root / args.dbt_models_dir

    if not migrations_dir.exists():
        print(f"Error: migrations directory not found: {migrations_dir}", file=sys.stderr)
        sys.exit(1)
    if not dbt_staging_dir.exists():
        print(f"Error: dbt staging directory not found: {dbt_staging_dir}", file=sys.stderr)
        sys.exit(1)

    mismatches = validate(migrations_dir, dbt_staging_dir, verbose=args.verbose)

    if not mismatches:
        print("All dbt staging models reference valid source columns.")
        sys.exit(0)

    print(f"\nColumn mismatch{'es' if len(mismatches) > 1 else ''} detected:\n")

    for m in mismatches:
        print(f"  {m.dbt_file}")
        print(f"    source: {m.source_table}")
        print(f"    missing columns: {', '.join(m.missing_columns)}")
        print(f"    available columns: {', '.join(sorted(m.available_columns - DLT_COLUMNS))}")

        staging_model_name = Path(m.dbt_file).stem
        if dbt_models_dir.exists():
            downstream = find_downstream_models(dbt_models_dir, staging_model_name)
            if downstream:
                print(f"    downstream impact: {' -> '.join(downstream)}")
        print()

    sys.exit(1)


if __name__ == "__main__":
    main()
