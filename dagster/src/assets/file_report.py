from typing import Callable, List, Literal, Optional, TypedDict

import requests

import dagster as dg
from generate_es_reports.constants import DEFAULT_REPORTS_YAML_PATH
from generate_es_reports.domain.report import BaseFileOutputConfig, ReportJobDefinition
from generate_es_reports.generator import generate_single_report
from generate_es_reports.io import BigQueryTableFetcher, load_report_jobs_from_yaml
from src.assets.dbt import _get_dbt_asset_key, _load_dbt_manifest
from src.core import Protoasset
from src.otel import _current_span_to_traceparent
from src.resources import (
    RESOURCE_KEY_DW_BQ,
    RESOURCE_KEY_FILE_REPORTS_BUCKET,
    BigQueryResource,
    GCSResource,
)


class ReportFile(TypedDict):
    """Represents a single file in a report."""

    type: Literal["csv", "txt", "xml"]
    path_in_bucket: str


class Report(TypedDict):
    """Represents a report with its metadata and associated files."""

    name: str
    norm: str
    files: List[ReportFile]


def get_dbt_asset_key_for_table(table_name: str) -> Optional[dg.AssetKey]:
    """Get the dbt asset key for a given table name.

    Searches the dbt manifest for a model matching the table name and returns
    its asset key based on the model's fqn (fully qualified name).
    """
    manifest = _load_dbt_manifest()

    for node_id, node in manifest["nodes"].items():
        if node.get("resource_type") == "model" and node["name"] == table_name:
            asset_key_path = _get_dbt_asset_key(manifest, node_id)
            return dg.AssetKey(asset_key_path)

    return None


def create_file_report_callable(
    report_job: ReportJobDefinition,
    file_config: BaseFileOutputConfig,
) -> Callable:
    """Create a callable that generates and uploads a single report file."""

    def _report_callable(
        context: dg.AssetExecutionContext,
        dw_bq: BigQueryResource,
        file_reports_bucket: GCSResource,
    ) -> None:
        table_fetcher = BigQueryTableFetcher(
            credentials_dict=dw_bq.get_credentials_dict(),
            dataset=dw_bq.get_dbt_dataset(),
        )

        def fetch_table(table_name: str):
            contents = table_fetcher.fetch_table_contents(table_name)
            return contents.fields, contents.records

        result = generate_single_report(
            fetch_table=fetch_table,
            upload_file=file_reports_bucket.upload_file,
            table_name=report_job.source_table_name,
            norm=report_job.norm,
            friendly_name=report_job.friendly_name,
            file_output_config=file_config,
            run_id=context.run_id,
            log=context.log.info,
        )

        # Extract just the path portion from the full GCS URL
        # gcs_path is "gs://bucket-name/path/to/file", we need just "path/to/file"
        gcs_path = result["gcs_path"]
        if gcs_path.startswith("gs://"):
            # Remove "gs://bucket-name/" prefix to get just the path in bucket
            path_in_bucket = "/".join(gcs_path.split("/")[3:])
        else:
            path_in_bucket = gcs_path

        report_file: ReportFile = {
            "type": result["file_type"],
            "path_in_bucket": path_in_bucket,
        }

        report: Report = {
            "name": result["friendly_name"],
            "norm": result["norm"],
            "files": [report_file],
        }

        context.add_output_metadata({"report": dg.MetadataValue.json(report)})

    return _report_callable


def generated_file_report_protoassets() -> List[Protoasset]:
    """Create protoassets for all enabled file reports from reports.yml.

    Each report job + file format combination becomes a separate asset.
    Assets depend on their corresponding dbt model.
    """
    protoassets = []
    report_jobs = load_report_jobs_from_yaml(DEFAULT_REPORTS_YAML_PATH)

    for report_job in report_jobs:
        for file_config in report_job.file_output_configs:
            asset_key = dg.AssetKey(
                [
                    "file_report",
                    f"{report_job.source_table_name}_{file_config.file_extension}",
                ]
            )

            dbt_dep = get_dbt_asset_key_for_table(report_job.source_table_name)
            deps = [dbt_dep] if dbt_dep else []

            protoassets.append(
                Protoasset(
                    key=asset_key,
                    callable=create_file_report_callable(report_job, file_config),
                    deps=deps,
                    required_resource_keys={
                        RESOURCE_KEY_FILE_REPORTS_BUCKET,
                        RESOURCE_KEY_DW_BQ,
                    },
                    automation_condition=None,
                    tags={
                        "category": "file_report",
                        "norm": report_job.norm,
                        "format": file_config.file_extension,
                    },
                )
            )

    return protoassets


def _get_file_report_asset_keys() -> List[dg.AssetKey]:
    """Get all file report asset keys."""
    return [p.key for p in generated_file_report_protoassets()]


def _extract_metadata_value(metadata: dict, key: str):
    """Extract the raw value from a Dagster MetadataValue object."""
    meta_value = metadata.get(key)
    if meta_value is None:
        return None
    # MetadataValue objects have a .value property containing the actual value
    return getattr(meta_value, "value", meta_value)


def inform_lana_of_new_reports(context: dg.AssetExecutionContext) -> None:
    """Collect metadata from all generated file reports and notify Lana system."""
    admin_server_url = dg.EnvVar("LANA_ADMIN_SERVER_URL").get_value()
    if not admin_server_url:
        raise ValueError(
            "LANA_ADMIN_SERVER_URL environment variable is not set. "
            "Please configure it to point to the Lana admin server."
        )
    webhook_url = f"{admin_server_url}/webhook/reports/sync"

    # Get traceparent from current span for distributed tracing
    headers = {}
    if traceparent := _current_span_to_traceparent():
        headers["traceparent"] = traceparent
        context.log.info(f"Sending traceparent: {traceparent}")

    try:
        context.log.info(f"Calling webhook: {webhook_url}")
        response = requests.post(webhook_url, headers=headers, timeout=30)
        response.raise_for_status()
        context.log.info(
            f"Successfully notified Lana system. Response: {response.status_code}"
        )
    except requests.exceptions.RequestException as e:
        context.log.error(f"Failed to notify Lana system: {e}")
        raise


def inform_lana_protoasset() -> Protoasset:
    """Create protoasset for informing Lana of new reports."""
    return Protoasset(
        key=dg.AssetKey("inform_lana_of_new_reports"),
        callable=inform_lana_of_new_reports,
        tags={"category": "notification"},
    )
