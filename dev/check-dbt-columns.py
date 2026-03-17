#!/usr/bin/env python3
"""
Validates dbt staging models against PostgreSQL migration schemas.

Checks for three kinds of issues:
  - SELECT * usage in staging models (columns must be listed explicitly)
  - References to source tables with no CREATE TABLE in migrations
  - References to columns that don't exist in the source table

Uses sqlglot to parse both migration DDL and dbt SQL (after stripping Jinja).

Usage:
    python dev/check-dbt-columns.py
    python dev/check-dbt-columns.py --verbose
"""

import argparse
import logging
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

import sqlglot
from sqlglot import exp

# Suppress sqlglot warnings for unsupported syntax (e.g. CREATE TRIGGER)
logging.getLogger("sqlglot").setLevel(logging.ERROR)

MIGRATIONS_DIR = "lana/app/migrations"
DBT_STAGING_DIR = "dagster/src/dbt_lana_dw/models/staging"

CORE_SETUP_MIGRATION = "20240517074612_core_setup.sql"
SQLGLOT_DIALECT = "bigquery"

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


@dataclass
class TableSchema:
    name: str
    columns: set[str] = field(default_factory=set)


class TableSchemas:
    """Map of table name -> known columns, built from migration files."""

    def __init__(self) -> None:
        self._tables: dict[str, TableSchema] = {}

    def __contains__(self, table_name: str) -> bool:
        return table_name in self._tables

    def columns_for(self, table_name: str) -> set[str]:
        return self._tables[table_name].columns

    def add(self, schema: TableSchema) -> None:
        self._tables[schema.name] = schema

    def add_columns_to_all(self, columns: set[str]) -> None:
        for schema in self._tables.values():
            schema.columns |= columns

    def items(self) -> list[tuple[str, set[str]]]:
        return [(s.name, s.columns) for s in self._tables.values()]

    def __len__(self) -> int:
        return len(self._tables)


def parse_create_table(sql: str) -> dict[str, set[str]]:
    """Extract column names from CREATE TABLE statements in migration SQL."""
    tables = {}
    for statement in sqlglot.parse(sql, dialect="postgres"):
        if not isinstance(statement, exp.Create):
            continue
        table_node = statement.find(exp.Table)
        if not table_node:
            continue
        columns = set()
        for col_def in statement.find_all(exp.ColumnDef):
            columns.add(col_def.name.lower())
        if columns:
            tables[table_node.name] = columns
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


def load_migration_schemas(migrations_dir: Path) -> TableSchemas:
    """Load column schemas from all migration files, latest wins."""
    schemas = TableSchemas()

    for migration_file in sorted(migrations_dir.glob("*.sql")):
        sql = migration_file.read_text()
        filename = migration_file.name

        if "_create_" in filename and "events_rollup" in filename:
            for name, cols in parse_create_table(sql).items():
                schemas.add(TableSchema(name=name, columns=cols))
        elif "_update_" in filename and "events_rollup" in filename:
            for name, cols in parse_create_table_from_comment(sql).items():
                schemas.add(TableSchema(name=name, columns=cols))
        elif filename == CORE_SETUP_MIGRATION:
            for name, cols in parse_create_table(sql).items():
                schemas.add(TableSchema(name=name, columns=cols))

    return schemas


_SOURCE_PATTERN = re.compile(
    r"\{\{\s*source\(\s*[\"']lana[\"']\s*,\s*[\"'](\w+)[\"']\s*\)\s*\}\}"
)


def extract_source_map(sql: str) -> dict[str, str]:
    """Extract a mapping of placeholder table names to real source table names.

    Each {{ source("lana", "TABLE") }} becomes __source__TABLE -> TABLE.
    """
    return {
        f"__source__{m.group(1)}": m.group(1) for m in _SOURCE_PATTERN.finditer(sql)
    }


def strip_jinja(sql: str) -> str:
    """Replace Jinja expressions with parseable SQL.

    source() calls become placeholder table names; all other Jinja is removed.
    """
    clean = _SOURCE_PATTERN.sub(lambda m: f"__source__{m.group(1)}", sql)
    clean = re.sub(r"\{\{.*?\}\}", "", clean, flags=re.DOTALL)
    clean = re.sub(r"\{%.*?%\}", "", clean, flags=re.DOTALL)
    return clean


def find_columns_from_source(node: exp.Expression) -> set[str]:
    """Walk a sqlglot AST node and collect all Column references."""
    columns = set()
    for col in node.find_all(exp.Column):
        columns.add(col.name.lower())
    return columns


@dataclass
class SourceRef:
    table: str
    columns: list[str]
    has_star: bool


def extract_source_columns_from_dbt(sql: str) -> list[SourceRef]:
    """Parse a dbt staging SQL file and extract source table + column references.

    Uses sqlglot to properly parse the SQL AST, handling CASE, nested functions, etc.
    """
    source_map = extract_source_map(sql)
    if not source_map:
        return []

    clean_sql = strip_jinja(sql)
    parsed = sqlglot.parse_one(clean_sql, dialect=SQLGLOT_DIALECT)

    results = []

    # Strategy: find CTEs or subqueries that SELECT FROM a source placeholder,
    # then collect all Column nodes within that SELECT.
    # Find SELECTs that read directly from a source placeholder
    source_selects: list[tuple[str, exp.Select]] = []
    for select in parsed.find_all(exp.Select):
        from_clause = select.find(exp.From)
        if not from_clause:
            continue
        table_node = from_clause.find(exp.Table)
        if not table_node:
            continue
        if table_node.name not in source_map:
            continue
        source_table = source_map[table_node.name]
        has_star = any(isinstance(e, exp.Star) for e in select.expressions)
        source_selects.append((source_table, select, has_star))

    # Extract column references from each source SELECT
    for source_table, select, has_star in source_selects:
        columns = []
        for col in select.find_all(exp.Column):
            col_name = col.name.lower()
            if col_name not in columns:
                columns.append(col_name)
        results.append(SourceRef(table=source_table, columns=columns, has_star=has_star))

    return results


@dataclass
class SelectStarUsage:
    dbt_file: str
    source_table: str


@dataclass
class MissingSource:
    dbt_file: str
    source_table: str


@dataclass
class ColumnMismatch:
    dbt_file: str
    source_table: str
    missing_columns: list[str]
    available_columns: set[str]


@dataclass
class DbtSourceRef:
    dbt_file: str
    source: SourceRef


def _collect_dbt_source_refs(dbt_staging_dir: Path) -> list[DbtSourceRef]:
    """Collect source references from all dbt staging files, skipping external tables."""
    results = []
    for dbt_file in sorted(dbt_staging_dir.rglob("*.sql")):
        sql = dbt_file.read_text()
        rel_path = str(dbt_file.relative_to(dbt_staging_dir.parents[2]))
        for ref in extract_source_columns_from_dbt(sql):
            if ref.table in SKIP_TABLES:
                continue
            results.append(DbtSourceRef(dbt_file=rel_path, source=ref))
    return results


def search_select_star_usages(source_refs: list[DbtSourceRef]) -> list[SelectStarUsage]:
    """Find dbt staging models that use SELECT * from a source table."""
    return [
        SelectStarUsage(dbt_file=entry.dbt_file, source_table=entry.source.table)
        for entry in source_refs
        if entry.source.has_star
    ]


def search_table_mismatches(
    schemas: TableSchemas, source_refs: list[DbtSourceRef]
) -> list[MissingSource]:
    """Find dbt staging models that reference source tables with no known schema."""
    return [
        MissingSource(dbt_file=entry.dbt_file, source_table=entry.source.table)
        for entry in source_refs
        if entry.source.table not in schemas
    ]


def search_column_mismatches(
    schemas: TableSchemas, source_refs: list[DbtSourceRef]
) -> list[ColumnMismatch]:
    """Find dbt staging models that reference columns missing from the source table."""
    mismatches = []
    for entry in source_refs:
        if entry.source.has_star or entry.source.table not in schemas:
            continue
        available = schemas.columns_for(entry.source.table)
        missing = [c for c in entry.source.columns if c not in available]
        if missing:
            mismatches.append(
                ColumnMismatch(
                    dbt_file=entry.dbt_file,
                    source_table=entry.source.table,
                    missing_columns=missing,
                    available_columns=available,
                )
            )
    return mismatches


def main():
    parser = argparse.ArgumentParser(
        description="Validate dbt staging models against PostgreSQL migration schemas"
    )
    parser.add_argument(
        "--migrations-dir",
        default=MIGRATIONS_DIR,
        help="Path to SQL migration files",
    )
    parser.add_argument(
        "--dbt-dir",
        default=DBT_STAGING_DIR,
        help="Path to dbt staging models directory",
    )
    parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parent.parent
    migrations_dir = repo_root / args.migrations_dir
    dbt_staging_dir = repo_root / args.dbt_dir

    if not migrations_dir.exists():
        print(f"Error: migrations directory not found: {migrations_dir}", file=sys.stderr)
        sys.exit(1)
    if not dbt_staging_dir.exists():
        print(f"Error: dbt staging directory not found: {dbt_staging_dir}", file=sys.stderr)
        sys.exit(1)

    schemas = load_migration_schemas(migrations_dir)

    if args.verbose:
        print(f"Loaded schemas for {len(schemas)} tables from migrations")
        for tbl, cols in sorted(schemas.items()):
            print(f"  {tbl}: {sorted(cols)}")
        print()

    # dlt adds metadata columns to every table
    schemas.add_columns_to_all(DLT_COLUMNS)

    source_refs = _collect_dbt_source_refs(dbt_staging_dir)
    star_usages = search_select_star_usages(source_refs)
    missing_sources = search_table_mismatches(schemas, source_refs)
    column_mismatches = search_column_mismatches(schemas, source_refs)

    has_issues = star_usages or missing_sources or column_mismatches

    if not has_issues:
        print("All dbt staging models reference valid source columns.")
        sys.exit(0)

    print()
    if star_usages:
        print(f"--- SELECT * not allowed ({len(star_usages)}) ---")
        print()
        for s in star_usages:
            print(f"  Model: {s.dbt_file}")
            print(f"  Source: {s.source_table}")
            print(f"  SELECT * prevents column validation; list columns explicitly.")
            print()

    if missing_sources:
        print(f"--- Missing source tables ({len(missing_sources)}) ---")
        print()
        for m in missing_sources:
            print(f"  Model: {m.dbt_file}")
            print(f"  Source: {m.source_table}")
            print(f"  No CREATE TABLE found in migrations for this source.")
            print()

    if column_mismatches:
        print(f"--- Column mismatches ({len(column_mismatches)}) ---")
        print()
        for m in column_mismatches:
            print(f"  Model:     {m.dbt_file}")
            print(f"  Source:    {m.source_table}")
            print(f"  Missing:   {', '.join(m.missing_columns)}")
            print(f"  Available: {', '.join(sorted(m.available_columns - DLT_COLUMNS))}")
            print()

    total = len(star_usages) + len(missing_sources) + len(column_mismatches)
    print(f"Found {total} issue{'s' if total > 1 else ''}.")
    sys.exit(1)


if __name__ == "__main__":
    main()
