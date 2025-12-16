from typing import Any, Callable

from generate_es_reports.domain.report import (
    BaseFileOutputConfig,
    TabularReportContents,
)
from generate_es_reports.io import encode_gcs_path


def generate_single_report(
    fetch_table: Callable[[str], tuple[list[str], list[dict[str, Any]]]],
    upload_file: Callable[[bytes, str, str], str],
    table_name: str,
    norm: str,
    friendly_name: str,
    file_output_config: BaseFileOutputConfig,
    run_id: str,
    log: Callable[[str], None] = print,
) -> dict[str, Any]:
    """Generate a single report file: fetch data, format it, upload.

    Args:
        fetch_table: Function to fetch table data. Takes table_name, returns (field_names, records).
        upload_file: Function to upload file. Takes (content_bytes, path, content_type), returns gcs_path.
        table_name: Name of the source table.
        norm: The regulatory norm this report belongs to.
        friendly_name: Human-friendly name for the report.
        file_output_config: Configuration for output format (CSV, XML, TXT).
        run_id: Unique identifier for this run (used in file path).
        log: Optional logging function (default: print).

    Returns:
        Dict with report metadata including GCS path.
    """
    extension = file_output_config.file_extension
    log(f"Generating {norm}/{friendly_name}.{extension}")

    # Fetch data
    log(f"Fetching table: {table_name}")
    field_names, records = fetch_table(table_name)
    log(f"Fetched {len(records)} rows with {len(field_names)} columns")

    # Convert to report format
    table_contents = TabularReportContents(field_names=field_names, records=records)
    storable_report = file_output_config.rows_to_report_output(table_contents)

    # Build file path
    path = f"reports/{run_id}/{norm}/{friendly_name}.{extension}"
    encoded_path = encode_gcs_path(path)

    # Upload
    log(f"Uploading to: {encoded_path}")
    gcs_path = upload_file(
        storable_report.content.encode("utf-8"),
        encoded_path,
        storable_report.content_type,
    )
    log(f"Uploaded to: {gcs_path}")

    return {
        "norm": norm,
        "friendly_name": friendly_name,
        "file_type": extension,
        "gcs_path": gcs_path,
        "row_count": len(records),
    }

