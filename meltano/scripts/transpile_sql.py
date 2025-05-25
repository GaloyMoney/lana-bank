#!/usr/bin/env python3
"""
transpile_sql.py
================
A tiny, dependency‑light wrapper around **sqlglot** that converts a SQL file
written for **BigQuery** into **DuckDB** dialect.

Designed to slot into your CI so you can assert that every **compiled dbt model**
parses for DuckDB _before_ merging.

Usage
-----
# Single file (prints to STDOUT)
python transpile_sql.py path/to/model.sql

# Write alongside the original
python transpile_sql.py path/to/model.sql -o path/to/model_duckdb.sql

# Skip errors and continue processing
python transpile_sql.py path/to/model.sql --skip-errors

# Use an exclusions file to skip specific files
python transpile_sql.py path/to/model.sql --exclusions-file path/to/exclusions.txt

# Bulk‑check every compiled model and fail on the first parse error
set -e
for f in target/compiled/**/*.sql; do
    python transpile_sql.py "$f" --show-diff >/dev/null
done

Dependencies
------------
    pip install sqlglot[rs]  # the optional Rust extensions speed things up

**Important**: feed this script *rendered* SQL only — e.g. the output of
`dbt compile --target bigquery`.  Jinja blocks/macros will break the parser.

Exit codes
~~~~~~~~~~
0 → success  | 1 → parse error  | 2 → other failure (e.g. file not found)
"""

import argparse
import difflib
import fnmatch
import os
import pathlib
import re
import sys
from typing import List, Tuple, Set, Optional

import sqlglot
from sqlglot.errors import ParseError


# ---------------------------------------------------------------------------
# Dialects — hard‑coded for now. If you need something else, tweak these two
# constants or expose them as CLI flags.
# ---------------------------------------------------------------------------
READ_DIALECT = "bigquery"
WRITE_DIALECT = "duckdb"

# Default path for an exclusions file (glob patterns, one per line)
DEFAULT_EXCLUSIONS_FILE = "meltano/scripts/sqlglot_exclusions.txt"


# ---------------------------------------------------------------------------
# Helper functions
# ---------------------------------------------------------------------------

def load_exclusions(exclusions_file: str) -> Set[str]:
    """Load file‑pattern globs to exclude from transpilation."""
    if not os.path.exists(exclusions_file):
        return set()

    with open(exclusions_file, "r", encoding="utf-8") as f:
        return {
            line.strip() for line in f
            if line.strip() and not line.lstrip().startswith("#")
        }


def is_excluded(file_path: pathlib.Path, exclusions: Set[str]) -> bool:
    """Return **True** if *file_path* matches any exclusion pattern."""
    normalised = str(file_path).replace(os.sep, "/")  # consistent slashes
    return any(fnmatch.fnmatch(normalised, pattern) for pattern in exclusions)


def transpile(sql: str, *, read: str = READ_DIALECT, write: str = WRITE_DIALECT, debug_mode: bool = False) -> str:
    """Return *pretty‑printed* DuckDB SQL produced by sqlglot."""
    parsed = sqlglot.parse_one(sql, read=read)
    if debug_mode: print(f"DEBUG: Original SQL to transpile:\n{sql[:500]}...", file=sys.stderr) # Print start of SQL

    # Drop the *catalog* (aka project ID) from three‑part identifiers — specific
    # to BigQuery's `project.dataset.table` grammar.
    for table in parsed.find_all(sqlglot.exp.Table):
        if table.args.get("catalog"):
            table.args.pop("catalog", None)
            if debug_mode: print(f"DEBUG: Dropped catalog from table: {table.sql(dialect=read)}", file=sys.stderr)

        # Transform schema if it's 'lana_dataset'
        current_schema_node = table.args.get("db")
        if current_schema_node and isinstance(current_schema_node, sqlglot.exp.Identifier):
            current_schema_name_str = current_schema_node.name
            if current_schema_name_str.lower() == "lana_dataset":
                table_name_for_debug = table.this.name if table.this and isinstance(table.this, sqlglot.exp.Identifier) else "N/A"
                if debug_mode: print(f"DEBUG: Transforming schema name '{current_schema_name_str}' to 'lana' for table '{table_name_for_debug}'", file=sys.stderr)
                new_schema_identifier = sqlglot.exp.Identifier(this="lana", quoted=current_schema_node.args.get('quoted', False))
                table.set("db", new_schema_identifier)

        if table.this:  # 'this' is the Identifier for the table name itself
            current_table_name_str = table.this.name
            transformed_name_str = current_table_name_str
            if transformed_name_str.startswith("public_"):
                transformed_name_str = transformed_name_str[len("public_"):]
            if transformed_name_str.endswith("_view"):
                transformed_name_str = transformed_name_str[:-len("_view")]
            if transformed_name_str != current_table_name_str:
                if debug_mode: print(f"DEBUG: Transforming table name '{current_table_name_str}' to '{transformed_name_str}'", file=sys.stderr)
                table.this.args['this'] = transformed_name_str

    # Transformation for _sdc_received_at to _sdc_batched_at in ORDER BY clauses within Window functions
    if debug_mode: print("DEBUG: Searching for Window expressions to modify ORDER BY...", file=sys.stderr)
    for window_expression in parsed.find_all(sqlglot.exp.Window):
        if debug_mode: print(f"DEBUG: Found Window expression: {window_expression.sql(dialect=read)}", file=sys.stderr)
        order_by_arg = window_expression.args.get("order")
        if order_by_arg and isinstance(order_by_arg, sqlglot.exp.Order):
            if debug_mode: print(f"DEBUG: Found Order clause: {order_by_arg.sql(dialect=read)}", file=sys.stderr)
            new_ordered_expressions_for_new_order_node = []
            modified_any_expression_in_order_clause = False
            for ordered_exp in order_by_arg.expressions:
                if debug_mode: print(f"DEBUG: Checking ordered expression: {ordered_exp.sql(dialect=read)}", file=sys.stderr)
                
                identifier_node = None
                identifier_name_str = ""
                is_identifier_type = False

                if isinstance(ordered_exp.this, sqlglot.exp.Identifier):
                    identifier_node = ordered_exp.this
                    identifier_name_str = identifier_node.name.lower()
                    is_identifier_type = True
                elif isinstance(ordered_exp.this, sqlglot.exp.Column) and isinstance(ordered_exp.this.this, sqlglot.exp.Identifier):
                    identifier_node = ordered_exp.this.this
                    identifier_name_str = identifier_node.name.lower()
                    is_identifier_type = True
                
                if debug_mode: print(f"DEBUG: Extracted identifier: {identifier_name_str if is_identifier_type else 'Not an Identifier/Column with Identifier'}", file=sys.stderr)

                if is_identifier_type and identifier_name_str == "_sdc_received_at":
                    if debug_mode: print("DEBUG: MATCH FOUND! Changing '_sdc_received_at' to '_sdc_batched_at'", file=sys.stderr)
                    new_identifier_ast_node = sqlglot.exp.Identifier(this="_sdc_batched_at", quoted=identifier_node.args.get('quoted', False))
                    
                    new_base_expression = None
                    if isinstance(ordered_exp.this, sqlglot.exp.Identifier):
                        new_base_expression = new_identifier_ast_node
                    elif isinstance(ordered_exp.this, sqlglot.exp.Column):
                        col_args = ordered_exp.this.args.copy()
                        col_args['this'] = new_identifier_ast_node
                        new_base_expression = sqlglot.exp.Column(**col_args)
                    
                    if new_base_expression:
                        new_ordered_exp = sqlglot.exp.Ordered(
                            this=new_base_expression,
                            desc=ordered_exp.args.get('desc', False),
                            nulls_first=ordered_exp.args.get('nulls_first')
                        )
                        new_ordered_expressions_for_new_order_node.append(new_ordered_exp)
                        modified_any_expression_in_order_clause = True
                    else:
                        if debug_mode: print("DEBUG: Failed to reconstruct base expression, keeping original.", file=sys.stderr)
                        new_ordered_expressions_for_new_order_node.append(ordered_exp) # Fallback
                else:
                    new_ordered_expressions_for_new_order_node.append(ordered_exp)
            
            if modified_any_expression_in_order_clause:
                if debug_mode: print("DEBUG: ORDER BY clause was modified. Reconstructing Order node.", file=sys.stderr)
                new_order_node = sqlglot.exp.Order(expressions=new_ordered_expressions_for_new_order_node)
                window_expression.set("order", new_order_node) 
            else:
                if debug_mode: print("DEBUG: ORDER BY clause was NOT modified.", file=sys.stderr)
        else:
            if debug_mode: print("DEBUG: No Order clause found in this Window expression or wrong type.", file=sys.stderr)

    # Transformation for safe.parse_json(event) to TRY_CAST(event AS JSON)
    if debug_mode: print("DEBUG: Inspecting all Func and Anonymous nodes for parse_json transformation...", file=sys.stderr)
    
    # Create a list of all candidate nodes first
    candidate_nodes = []
    for node_type_class in [sqlglot.exp.Func, sqlglot.exp.Anonymous]:
        for node in parsed.find_all(node_type_class):
            candidate_nodes.append(node)

    for func_call_node in candidate_nodes:
        node_sql_original_dialect = func_call_node.sql(dialect=read)
        if debug_mode: print(f"DEBUG CANDIDATE: Type={type(func_call_node).__name__}, SQL='{node_sql_original_dialect}'", file=sys.stderr)

        # Default to not transforming
        transform_this_node = False
        argument_node_for_transform = None

        if isinstance(func_call_node, sqlglot.exp.Func) and \
           hasattr(func_call_node, 'this') and isinstance(func_call_node.this, sqlglot.exp.Identifier) and \
           func_call_node.this.name.lower() == 'parse_json' and \
           hasattr(func_call_node, 'expressions') and len(func_call_node.expressions) == 1:
            
            transform_this_node = True
            argument_node_for_transform = func_call_node.expressions[0]
            if debug_mode: print(f"DEBUG TRANSFORM TARGET (Func): Found 'parse_json'. Assuming original safe.parse_json.", file=sys.stderr)

        elif isinstance(func_call_node, sqlglot.exp.Anonymous) and \
             hasattr(func_call_node, 'this') and isinstance(func_call_node.this, str) and \
             func_call_node.this.lower() == 'parse_json' and \
             hasattr(func_call_node, 'expressions') and len(func_call_node.expressions) == 1:

            transform_this_node = True
            argument_node_for_transform = func_call_node.expressions[0]
            if debug_mode: print(f"DEBUG TRANSFORM TARGET (Anonymous): Found 'parse_json'. Assuming original safe.parse_json.", file=sys.stderr)

        if transform_this_node and argument_node_for_transform:
            argument_sql_str = argument_node_for_transform.sql(dialect=read)
            # print(f"DEBUG: Argument for transform: {argument_sql_str}", file=sys.stderr) # Added for clarity
            try_cast_expr_str = f"TRY_CAST({argument_sql_str} AS JSON)"
            try:
                # Parse the replacement string using the TARGET dialect (duckdb)
                replacement_node = sqlglot.parse_one(try_cast_expr_str, read=write)
                if replacement_node and func_call_node.parent:
                    func_call_node.replace(replacement_node)
                    if debug_mode: print(f"DEBUG TRANSFORMED: Replaced '{node_sql_original_dialect}' with '{replacement_node.sql(dialect=write)}'", file=sys.stderr)
            except Exception as e_parse_replace:
                if debug_mode: print(f"DEBUG TRANSFORM ERROR: for '{node_sql_original_dialect}': {e_parse_replace}", file=sys.stderr)
        # else:
            # Optional: print if a candidate didn't match the specific transform conditions
            # print(f"DEBUG CANDIDATE SKIPPED (not matching parse_json Func criteria): Type={type(func_call_node).__name__}, SQL='{node_sql_original_dialect}'", file=sys.stderr)

    duckdb_sql = parsed.sql(dialect=write, pretty=True)
    if debug_mode: print(f"DEBUG: SQL after sqlglot render (before final string replace):\n{duckdb_sql[:500]}...", file=sys.stderr) # New debug

    # Final cleanup: explicitly remove "safe.TRY_CAST" if sqlglot re-added "safe."
    cleaned_duckdb_sql = duckdb_sql.replace("safe.TRY_CAST", "TRY_CAST")
    if cleaned_duckdb_sql != duckdb_sql:
        if debug_mode: print(f"DEBUG: Performed final string replacement of 'safe.TRY_CAST'", file=sys.stderr)
    
    final_sql_for_unnest_processing = cleaned_duckdb_sql

    pattern  = r"""
        UNNEST\s*\(
            (?P<array>[^)]+)
        \)\s+
        WITH\s+ORDINALITY\s+
        AS\s+
        (?P<alias>\w+)
        \s*\(
            (?P<elem>\w+)\s*,\s*(?P<idx>\w+)
        \)
    """
    replacement = (
        r"(SELECT {elem}, ROW_NUMBER() OVER () - 1 AS {idx} "
        r"FROM UNNEST({array}) AS {elem}) AS {alias}"
    )
    duckdb_sql_new = re.sub(pattern, lambda m: replacement.format(**m.groupdict()),
                            final_sql_for_unnest_processing, flags=re.I | re.X)
    
    if duckdb_sql_new != final_sql_for_unnest_processing:
         if debug_mode: print(f"DEBUG: UNNEST regex transformation was applied.", file=sys.stderr)
    else:
         if debug_mode: print(f"DEBUG: UNNEST regex transformation was NOT applied (no match).", file=sys.stderr)

    return duckdb_sql_new


def show_diff(original: str, converted: str, lhs: str, rhs: str) -> None:
    """Print a unified diff between *original* and *converted* SQL."""
    diff = difflib.unified_diff(
        original.splitlines(keepends=True),
        converted.splitlines(keepends=True),
        fromfile=lhs,
        tofile=rhs,
    )
    sys.stdout.writelines(diff)


def find_error_line_and_context(sql: str, error_msg: str) -> Tuple[int, str, str]:
    """Pull out the line & column mentioned in sqlglot's error message."""
    match = re.search(r"Line (\d+), Col: (\d+)", error_msg)
    if not match:
        return 0, "", ""  # couldn't parse it

    line_num, col_num = int(match.group(1)), int(match.group(2))
    lines = sql.splitlines()
    if not (0 < line_num <= len(lines)):
        return 0, "", ""

    context_line = lines[line_num - 1]
    indicator = " " * (col_num - 1) + "^"
    return line_num, context_line, indicator


# ---------------------------------------------------------------------------
# File‑processing pipeline
# ---------------------------------------------------------------------------

def process_single_file(
    input_file: pathlib.Path,
    output_file: Optional[pathlib.Path],
    args: argparse.Namespace,
    exclusions: Set[str],
) -> bool:
    """Transpile one file. Return **True** on success, **False** on failure."""
    if is_excluded(input_file, exclusions):
        if args.debug:
            print(f"⚠️  Skipping excluded file: {input_file}", file=sys.stderr)
        if output_file:
            # Just copy it byte‑for‑byte to the destination
            output_file.parent.mkdir(parents=True, exist_ok=True)
            output_file.write_text(input_file.read_text(encoding="utf-8"), encoding="utf-8")
        return True

    try:
        original = input_file.read_text(encoding="utf-8")
    except FileNotFoundError:
        print(f"❌ File not found: {input_file}", file=sys.stderr)
        return False
    except Exception as exc:
        print(f"❌ Error reading {input_file}: {exc}", file=sys.stderr)
        return False

    try:
        converted = transpile(original, debug_mode=args.debug)
    except ParseError as exc:
        msg = str(exc)
        print(f"❌ Parse error in {input_file}: {msg}", file=sys.stderr)

        if args.debug:
            ln, ctx, caret = find_error_line_and_context(original, msg)
            if ln:
                print(f"\nError at line {ln}:", file=sys.stderr)
                print(f"  {ctx}", file=sys.stderr)
                print(f"  {caret}", file=sys.stderr)
        if args.skip_errors:
            print(f"⚠️  Using original SQL (skip‑errors)", file=sys.stderr)
            converted = original
        else:
            return False
    except Exception as exc:
        print(f"❌ Unexpected error in {input_file}: {exc}", file=sys.stderr)
        return False

    if args.show_diff:
        show_diff(original, converted, str(input_file), str(output_file or "<duckdb>"))

    if output_file:
        try:
            output_file.parent.mkdir(parents=True, exist_ok=True)
            output_file.write_text(converted, encoding="utf-8")
        except Exception as exc:
            print(f"❌ Error writing {output_file}: {exc}", file=sys.stderr)
            return False
    else:
        print(converted)

    return True


# ---------------------------------------------------------------------------
# Main entry‑point
# ---------------------------------------------------------------------------

def main() -> None:
    parser = argparse.ArgumentParser(
        description="Transpile BigQuery SQL to DuckDB using sqlglot",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument("path", type=pathlib.Path,
                        help="Input .sql file or directory (compiled)")
    parser.add_argument("-o", "--out", type=pathlib.Path,
                        help="Where to write the converted SQL. If omitted, prints to STDOUT. \n"
                             "If the input is a directory, this must also be a directory.")
    parser.add_argument("--show-diff", action="store_true",
                        help="Show a unified diff between the original and converted SQL")
    parser.add_argument("--skip-errors", action="store_true",
                        help="Continue processing even when parse errors occur")
    parser.add_argument("--debug", action="store_true",
                        help="Print detailed debug information for errors")
    parser.add_argument("--exclusions-file", type=str,
                        default=DEFAULT_EXCLUSIONS_FILE,
                        help="Path to a file containing glob patterns of files to exclude")

    args = parser.parse_args()

    exclusions = load_exclusions(args.exclusions_file)

    # Gather files to process -------------------------------------------------
    if args.path.is_file():
        files = [args.path]
        single_file_mode = True
        if args.out and args.out.is_dir():
            parser.error("When the input is a single file, --out cannot be a directory.")
    elif args.path.is_dir():
        files = sorted(args.path.rglob("*.sql"))
        single_file_mode = False
        if not files:
            print(f"🤷 No .sql files found under {args.path}", file=sys.stderr)
            sys.exit(0)
        if args.out and args.out.is_file():
            parser.error("When the input is a directory, --out must be a directory (or omitted).")
        if args.out:
            args.out.mkdir(parents=True, exist_ok=True)
    else:
        parser.error("Input path is neither a file nor a directory.")
        return  # pragma: no cover

    overall_rc = 0  # 0=success, 1=parse error, 2=other

    for in_file in files:
        out_file = None
        if args.out:
            if single_file_mode:
                out_file = args.out
            else:
                out_file = args.out / in_file.relative_to(args.path)

        if args.debug:
            print(f"Processing: {in_file} -> {out_file or 'STDOUT'}", file=sys.stderr)

        ok = process_single_file(in_file, out_file, args, exclusions)
        if not ok and not args.skip_errors:
            sys.exit(1)
        if not ok and overall_rc == 0:
            overall_rc = 1

    if overall_rc != 0 and args.skip_errors:
        print("\n⚠️  Finished with errors (skipped).", file=sys.stderr)

    sys.exit(overall_rc)


if __name__ == "__main__":
    main()
