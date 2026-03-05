"""CI check: validate that expected_columns in reports.yml match dbt manifest columns.

For reports with expected_columns declared, verifies that the dbt model's
columns (from manifest.json) match exactly. For supports_as_of reports,
as_of_date is excluded from the manifest columns before comparison since
it is filtered out at query time.

Usage:
    python -m generate_es_reports.validate_report_columns <manifest_path>
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

import yaml

REPORTS_YAML_PATH = Path(__file__).parent / "reports.yml"


def validate(manifest_path: Path) -> list[str]:
    with open(REPORTS_YAML_PATH, "r") as f:
        reports_config = yaml.safe_load(f)

    with open(manifest_path, "r") as f:
        manifest = json.load(f)

    model_columns: dict[str, list[str]] = {}
    for node in manifest["nodes"].values():
        if node.get("resource_type") == "model" and node.get("columns"):
            model_columns[node["name"]] = list(node["columns"].keys())

    errors: list[str] = []
    for report_job in reports_config["report_jobs"]:
        expected = report_job.get("expected_columns")
        if expected is None:
            continue

        source_table = report_job["source_table"]
        manifest_cols = model_columns.get(source_table)
        if manifest_cols is None:
            errors.append(
                f"{report_job['norm']}/{report_job['id']}: "
                f"source_table '{source_table}' has no columns declared "
                f"in dbt manifest (add columns to the model's .yml file)"
            )
            continue

        if report_job.get("supports_as_of", False):
            manifest_cols = [c for c in manifest_cols if c != "as_of_date"]

        if manifest_cols != expected:
            errors.append(
                f"{report_job['norm']}/{report_job['id']}: "
                f"column mismatch for '{source_table}': "
                f"reports.yml expects {expected}, "
                f"dbt manifest has {manifest_cols}"
            )

    return errors


def main() -> None:
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} <manifest.json path>", file=sys.stderr)
        sys.exit(2)

    manifest_path = Path(sys.argv[1])
    if not manifest_path.exists():
        print(f"Manifest not found: {manifest_path}", file=sys.stderr)
        sys.exit(2)

    errors = validate(manifest_path)
    if errors:
        print("Column validation failed:", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        sys.exit(1)

    print("Column validation passed.")


if __name__ == "__main__":
    main()
