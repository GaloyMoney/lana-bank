from concurrent.futures import ThreadPoolExecutor, as_completed
from typing import Dict, List, Literal, Optional, Tuple, TypedDict

import requests

import dagster as dg
from generate_es_reports.constants import DEFAULT_REPORTS_YAML_PATH
from generate_es_reports.domain.report import BaseFileOutputConfig, ReportJobDefinition
from generate_es_reports.generator import generate_single_report
from generate_es_reports.io import BigQueryTableFetcher, load_report_jobs_from_yaml
from src.assets.dbt import _get_dbt_asset_key, _load_dbt_manifest
from src.core import COLD_START_CONDITION, Protoasset
from src.otel import (
    _current_span_to_traceparent,
    get_asset_span_context_and_attrs,
    tracer,
)
from src.resources import (
    RESOURCE_KEY_DW_BQ,
    RESOURCE_KEY_FILE_REPORTS_BUCKET,
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


def _build_file_report_specs_and_lookup() -> Tuple[
    List[dg.AssetSpec],
    Dict[dg.AssetKey, Tuple[ReportJobDefinition, BaseFileOutputConfig]],
]:
    """Build AssetSpec list and lookup dict from reports YAML config.

    Returns:
        Tuple of (specs, asset_key_to_job_config) where specs is the list of
        AssetSpecs and asset_key_to_job_config maps each asset key to its
        (report_job, file_config) pair.
    """
    report_jobs = load_report_jobs_from_yaml(DEFAULT_REPORTS_YAML_PATH)

    specs: List[dg.AssetSpec] = []
    asset_key_to_job_config: Dict[
        dg.AssetKey, Tuple[ReportJobDefinition, BaseFileOutputConfig]
    ] = {}

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

            specs.append(
                dg.AssetSpec(
                    key=asset_key,
                    deps=deps,
                    tags={
                        "category": "file_report",
                        "norm": report_job.norm,
                        "format": file_config.file_extension,
                    },
                    automation_condition=COLD_START_CONDITION,
                )
            )

            asset_key_to_job_config[asset_key] = (report_job, file_config)

    return specs, asset_key_to_job_config


def _process_single_asset(
    asset_key: dg.AssetKey,
    report_job: ReportJobDefinition,
    file_config: BaseFileOutputConfig,
    credentials_dict: dict,
    dataset: str,
    upload_file,
    run_id: str,
    log,
    otel_parent_ctx,
) -> Tuple[dg.AssetKey, dict]:
    """Fetch, format, and upload a single report file.

    Runs in a worker thread. Creates its own BQ client to avoid
    sharing connections across threads.
    """
    asset_key_str = asset_key.to_user_string()

    with tracer.start_as_current_span(
        f"file_report_{asset_key_str}",
        context=otel_parent_ctx,
    ) as span:
        span.set_attribute("asset.name", asset_key_str)
        span.set_attribute("report.norm", report_job.norm)
        span.set_attribute("report.format", file_config.file_extension)

        fetcher = BigQueryTableFetcher(
            credentials_dict=credentials_dict,
            dataset=dataset,
        )

        def fetch_table(table_name: str):
            contents = fetcher.fetch_table_contents(table_name)
            return (contents.fields, contents.records)

        result = generate_single_report(
            fetch_table=fetch_table,
            upload_file=upload_file,
            table_name=report_job.source_table_name,
            norm=report_job.norm,
            friendly_name=report_job.friendly_name,
            file_output_config=file_config,
            run_id=run_id,
            log=log,
        )
        return asset_key, result


def _build_materialize_result(
    asset_key: dg.AssetKey, result: dict
) -> dg.MaterializeResult:
    """Build a MaterializeResult from a report generation result."""
    gcs_path = result["gcs_path"]
    if gcs_path.startswith("gs://"):
        path_in_bucket = "/".join(gcs_path.split("/")[3:])
    else:
        path_in_bucket = gcs_path

    report: Report = {
        "name": result["friendly_name"],
        "norm": result["norm"],
        "files": [
            {
                "type": result["file_type"],
                "path_in_bucket": path_in_bucket,
            }
        ],
    }

    return dg.MaterializeResult(
        asset_key=asset_key,
        metadata={
            "report": dg.MetadataValue.json(report),
        },
    )


def create_file_report_multi_asset():
    """Create a single multi_asset for all file report generation.

    Uses can_subset=True so individual reports can be materialized independently.
    Each report is processed in its own thread (fetch → format → upload).
    """
    specs, asset_key_to_job_config = _build_file_report_specs_and_lookup()

    @dg.multi_asset(
        specs=specs,
        can_subset=True,
        required_resource_keys={RESOURCE_KEY_FILE_REPORTS_BUCKET, RESOURCE_KEY_DW_BQ},
    )
    def file_report_assets(context: dg.AssetExecutionContext):
        from opentelemetry import context as otel_context

        dw_bq = context.resources.dw_bq
        file_reports_bucket = context.resources.file_reports_bucket
        credentials_dict = dw_bq.get_credentials_dict()
        dataset = dw_bq.get_dbt_dataset()

        selected_keys = [key.to_user_string() for key in context.selected_asset_keys]
        parent_ctx, batch_attrs = get_asset_span_context_and_attrs(
            context, "file_report_assets"
        )
        batch_attrs["file_report.asset_count"] = str(len(selected_keys))
        batch_attrs["file_report.assets"] = str(selected_keys)

        with tracer.start_as_current_span(
            "file_report_batch", context=parent_ctx
        ) as batch_span:
            for key, value in batch_attrs.items():
                batch_span.set_attribute(key, value)

            batch_ctx = otel_context.get_current()

            failed_assets: list[tuple[dg.AssetKey, Exception]] = []

            with ThreadPoolExecutor(max_workers=16) as pool:
                futures = {
                    pool.submit(
                        _process_single_asset,
                        asset_key=k,
                        report_job=asset_key_to_job_config[k][0],
                        file_config=asset_key_to_job_config[k][1],
                        credentials_dict=credentials_dict,
                        dataset=dataset,
                        upload_file=file_reports_bucket.upload_file,
                        run_id=context.run_id,
                        log=context.log.info,
                        otel_parent_ctx=batch_ctx,
                    ): k
                    for k in context.selected_asset_keys
                }
                for f in as_completed(futures):
                    asset_key = futures[f]
                    try:
                        _, result = f.result()
                        yield _build_materialize_result(asset_key, result)
                    except Exception as e:
                        context.log.error(
                            f"Failed to generate report for {asset_key.to_user_string()}: {e}"
                        )
                        failed_assets.append((asset_key, e))

            if failed_assets:
                names = [k.to_user_string() for k, _ in failed_assets]
                raise RuntimeError(
                    f"{len(failed_assets)} report(s) failed: {', '.join(names)}"
                )

    return file_report_assets


def _get_file_report_asset_keys() -> List[dg.AssetKey]:
    """Get all file report asset keys."""
    specs, _ = _build_file_report_specs_and_lookup()
    return [spec.key for spec in specs]


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
