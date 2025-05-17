#!/usr/bin/env python3
"""
transpile_sql.py
================
A tiny, dependency‑light wrapper around **sqlglot** that converts a SQL file
written for **BigQuery** into **DuckDB** dialect.

Designed to slot into your CI so you can assert that every **compiled dbt model**
parses for Snowflake _before_ merging.

Usage
-----
# Single file (prints to STDOUT)
python transpile_sql.py path/to/model.sql

# Write alongside the original
python transpile_sql.py path/to/model.sql -o path/to/model_snowflake.sql

# Skip errors and continue processing
python transpile_sql.py path/to/model.sql --skip-errors

# Use exclusions file to skip specific files
python transpile_sql.py path/to/model.sql --exclusions-file path/to/exclusions.txt

# Bulk check all compiled models and fail on first parse error
set -e
for f in target/compiled/**/*.sql; do
    python transpile_sql.py "$f" --show-diff >/dev/null
done

Dependencies
------------
    pip install sqlglot[rs]  # the optional Rust extensions speed things up

**Important**: feed this script _rendered_ SQL only — e.g. the output of
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
import sys
from typing import List, Tuple, Set, Optional

import sqlglot
from sqlglot.errors import ParseError


READ_DIALECT = "bigquery"
WRITE_DIALECT = "duckdb"

DEFAULT_EXCLUSIONS_FILE = "meltano/scripts/sqlglot_exclusions.txt"


def load_exclusions(exclusions_file: str) -> Set[str]:
    """Load file patterns to exclude from transpilation."""
    if not os.path.exists(exclusions_file):
        return set()
    
    exclusions = set()
    with open(exclusions_file, "r") as f:
        for line in f:
            line = line.strip()
            if line and not line.startswith("#"):
                exclusions.add(line)
    return exclusions


def is_excluded(file_path: pathlib.Path, exclusions: Set[str]) -> bool:
    """Check if a file should be excluded from transpilation."""
    # Convert to Unix-style paths for consistent pattern matching
    normalized_path = str(file_path).replace(os.sep, "/")
    
    for pattern in exclusions:
        if fnmatch.fnmatch(normalized_path, pattern):
            return True
    return False


def transpile(sql: str, read: str = READ_DIALECT, write: str = WRITE_DIALECT) -> str:
    """Return *pretty‑printed* Snowflake SQL produced by sqlglot."""
    # First, parse the SQL from the read dialect
    parsed_expression = sqlglot.parse_one(sql, read=read)

    # Remove the catalog (project ID) from all table references
    # This assumes the catalog is the first part of a three-part identifier
    for table_exp in parsed_expression.find_all(sqlglot.exp.Table):
        if table_exp.args.get("catalog"):
            table_exp.args.pop("catalog", None) # Remove catalog part
            # If a schema (dataset) exists, it becomes the new 'db' (which sqlglot uses for schema)
            # if table_exp.args.get("db"):
            #     pass # Schema is already the db part
            # If there was no schema (only catalog.table), then db might be empty or table itself
            # This part is a bit tricky, sqlglot handles it based on how it tokenizes
            # The key is that `catalog` is removed. If schema was `db`, it remains.
            # If it was `catalog.table`, after removing catalog, it becomes just `table`.
            # If it was `catalog.schema.table`, it becomes `schema.table`.

    # Transpile to the write dialect and pretty print
    return parsed_expression.sql(dialect=write, pretty=True)


def show_diff(original: str, converted: str, lhs: str, rhs: str) -> None:
    """Print a unified diff between the original and converted SQL."""
    diff: List[str] = difflib.unified_diff(
        original.splitlines(keepends=True),
        converted.splitlines(keepends=True),
        fromfile=lhs,
        tofile=rhs,
    )
    sys.stdout.writelines(diff)


def find_error_line_and_context(sql: str, error_msg: str) -> Tuple[int, str, str]:
    """
    Extract line number, context, and error position from error message.
    
    Returns:
        Tuple of (line_number, context_line, error_indicator)
    """
    lines = sql.splitlines()
    
    # Try to extract line and column information from the error message
    line_col_info = None
    
    # Look for "Line X, Col: Y" pattern in error message
    import re
    match = re.search(r"Line (\d+), Col: (\d+)", error_msg)
    if match:
        line_num = int(match.group(1))
        col_num = int(match.group(2))
        
        if 0 <= line_num - 1 < len(lines):
            context_line = lines[line_num - 1]
            
            # Create an error indicator pointing to the error position
            error_indicator = ' ' * (col_num - 1) + '^'
            
            return line_num, context_line, error_indicator
    
    # Fallback to just returning the error message
    return 0, "", ""


def process_single_file(
    input_file: pathlib.Path, 
    output_file: Optional[pathlib.Path], 
    args: argparse.Namespace, 
    exclusions: Set[str]
) -> bool:
    """Processes a single SQL file. Returns True on success, False on failure."""
    if is_excluded(input_file, exclusions):
        if args.debug:
            print(f"⚠️ Skipping transpilation for excluded file: {input_file}", file=sys.stderr)
        if output_file:
            # If an output file is specified, just copy the original
            try:
                original = input_file.read_text(encoding="utf-8")
                output_file.parent.mkdir(parents=True, exist_ok=True) # Ensure output dir exists
                output_file.write_text(original, encoding="utf-8")
            except FileNotFoundError:
                print(f"❌ File not found: {input_file}", file=sys.stderr)
                return False # Indicate failure
            except Exception as e:
                print(f"❌ Error copying excluded file {input_file} to {output_file}: {e}", file=sys.stderr)
                return False
        return True # Indicate success (skipped)

    try:
        original = input_file.read_text(encoding="utf-8")
    except FileNotFoundError:
        print(f"❌ File not found: {input_file}", file=sys.stderr)
        return False # Indicate failure
    except Exception as e:
        print(f"❌ Error reading file {input_file}: {e}", file=sys.stderr)
        return False


    try:
        converted = transpile(original)
    except ParseError as e:
        error_msg = str(e)
        print(f"❌ Parse error in {input_file}: {error_msg}", file=sys.stderr)
        
        if args.debug:
            line_num, context_line, error_indicator = find_error_line_and_context(original, error_msg)
            if line_num > 0:
                print(f"\nError detected at line {line_num}:", file=sys.stderr)
                print(f"  {context_line}", file=sys.stderr)
                print(f"  {error_indicator}", file=sys.stderr)
                
                start_line = max(0, line_num - 3)
                end_line = min(len(original.splitlines()), line_num + 2)
                print("\nContext:", file=sys.stderr)
                for i, line_content in enumerate(original.splitlines()[start_line:end_line], start=start_line + 1):
                    prefix = "→ " if i == line_num else "  "
                    print(f"{prefix}{i}: {line_content}", file=sys.stderr)
        
        if args.skip_errors:
            print(f"⚠️ Skipping transpilation for {input_file} due to parse error, using original SQL", file=sys.stderr)
            converted = original # Use original content if skipping errors
        else:
            return False # Indicate failure
    except Exception as e:
        print(f"❌ Unexpected error transpiling {input_file}: {e}", file=sys.stderr)
        return False


    if args.show_diff:
        show_diff(original, converted, str(input_file), str(output_file or "<snowflake>"))

    if output_file:
        try:
            output_file.parent.mkdir(parents=True, exist_ok=True) # Ensure output dir exists
            output_file.write_text(converted, encoding="utf-8")
        except Exception as e:
            print(f"❌ Error writing to output file {output_file}: {e}", file=sys.stderr)
            return False
    else:
        print(converted)
    
    return True # Indicate success


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Transpile BigQuery SQL to Snowflake using sqlglot",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument("path", type=pathlib.Path, help="Input .sql file or directory (compiled)")
    parser.add_argument("-o", "--out", type=pathlib.Path, help="Where to write the converted SQL. If omitted, prints to STDOUT. If input is a directory, this must also be a directory.")
    parser.add_argument("--show-diff", action="store_true", help="Show a unified diff between the original and converted SQL")
    parser.add_argument("--skip-errors", action="store_true", help="Continue processing even when parse errors occur")
    parser.add_argument("--debug", action="store_true", help="Print detailed debug information for errors")
    parser.add_argument("--exclusions-file", type=str, default=DEFAULT_EXCLUSIONS_FILE, 
                      help="Path to a file containing glob patterns of files to exclude")

    args = parser.parse_args()

    exclusions = load_exclusions(args.exclusions_file)
    
    files_to_process: List[pathlib.Path] = []
    is_single_file_mode = False

    if args.path.is_file():
        files_to_process = [args.path]
        is_single_file_mode = True
        if args.out and args.out.is_dir():
            print(f"❌ Error: If input path '{args.path}' is a file, output path '--out {args.out}' cannot be a directory.", file=sys.stderr)
            sys.exit(2)
    elif args.path.is_dir():
        files_to_process = sorted(list(args.path.rglob("*.sql"))) # Sort for consistent processing order
        if not files_to_process:
            print(f"🤷 No .sql files found in directory: {args.path}", file=sys.stderr)
            sys.exit(0)
        if args.out and args.out.is_file():
            print(f"❌ Error: If input path '{args.path}' is a directory, output path '--out {args.out}' must also be a directory (or omitted).", file=sys.stderr)
            sys.exit(2)
        if args.out:
            args.out.mkdir(parents=True, exist_ok=True) # Ensure base output dir exists
    else:
        print(f"❌ Input path is not a valid file or directory: {args.path}", file=sys.stderr)
        sys.exit(2)

    overall_rc = 0 # 0 for success, 1 for parse error, 2 for other

    for input_file in files_to_process:
        output_file_path: Optional[pathlib.Path] = None
        if args.out:
            if is_single_file_mode:
                output_file_path = args.out
            else: # Directory mode
                relative_path = input_file.relative_to(args.path)
                output_file_path = args.out / relative_path
        
        print(f"Processing: {input_file} -> {output_file_path or 'STDOUT'}", file=sys.stderr if args.debug else sys.stdout)

        success = process_single_file(input_file, output_file_path, args, exclusions)
        
        if not success:
            if overall_rc == 0: # First error
                 overall_rc = 1 # Default to parse error type
                 # Check if it was a file not found error from process_single_file for a more specific rc
                 # This part is tricky as process_single_file returns bool. Assume 1 for now.
            if not args.skip_errors:
                sys.exit(1) # Exit on first error if not skipping

    if overall_rc != 0 and args.skip_errors: # If there were errors but we skipped them
        print(f"\n⚠️ Finished with errors (skipped). Final exit code reflects first error type: {overall_rc}", file=sys.stderr)
        sys.exit(overall_rc)
    
    sys.exit(overall_rc) # Will be 0 if all successful


if __name__ == "__main__":
    main()
