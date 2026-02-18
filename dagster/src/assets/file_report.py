import time
import traceback
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


def create_file_report_multi_asset():
    """Create a single multi_asset for all file report generation.

    Uses can_subset=True so individual reports can be materialized independently.
    Shares BigQuery fetches across formats of the same report within a single run.
    """
    specs, asset_key_to_job_config = _build_file_report_specs_and_lookup()

    @dg.multi_asset(
        specs=specs,
        can_subset=True,
        required_resource_keys={RESOURCE_KEY_FILE_REPORTS_BUCKET, RESOURCE_KEY_DW_BQ},
    )
    def file_report_assets(context: dg.AssetExecutionContext):
        log = context.log
        dw_bq = context.resources.dw_bq
        file_reports_bucket = context.resources.file_reports_bucket

        dataset = dw_bq.get_dbt_dataset()
        creds_dict = dw_bq.get_credentials_dict()
        project_id = creds_dict["project_id"]

        table_fetcher = BigQueryTableFetcher(
            credentials_dict=creds_dict,
            dataset=dataset,
        )

        # [DIAG] Log BQ context
        log.info(f"[DIAG] BigQuery project={project_id} dataset={dataset}")

        # [DIAG] Probe: list all report_* and static_* tables/views in dataset
        try:
            from google.cloud import bigquery as bq_module
            from google.oauth2 import service_account as sa_module

            probe_creds = sa_module.Credentials.from_service_account_info(creds_dict)
            probe_client = bq_module.Client(
                project=project_id, credentials=probe_creds
            )
            probe_query = (
                f"SELECT table_name, table_type "
                f"FROM `{project_id}.{dataset}.INFORMATION_SCHEMA.TABLES` "
                f"WHERE table_name LIKE 'report\\_%' "
                f"OR table_name LIKE 'static\\_%' "
                f"OR table_name LIKE 'seed\\_%' "
                f"ORDER BY table_name"
            )
            probe_job = probe_client.query(probe_query)
            probe_rows = list(probe_job.result())
            log.info(
                f"[DIAG] INFORMATION_SCHEMA probe: found {len(probe_rows)} "
                f"report_*/static_*/seed_* tables/views in {dataset}"
            )
            for row in probe_rows:
                log.info(f"[DIAG]   {row['table_name']} ({row['table_type']})")
        except Exception as e:
            log.warning(f"[DIAG] INFORMATION_SCHEMA probe failed: {e}")

        # Cache fetched tables to avoid duplicate BigQuery reads
        # when multiple formats share the same source table
        table_cache: Dict[str, Tuple] = {}
        # [DIAG] Track fetch timing
        fetch_times: Dict[str, float] = {}

        def fetch_table_cached(table_name: str):
            if table_name in table_cache:
                log.info(f"[DIAG] Cache HIT for {table_name}")
                return table_cache[table_name]
            log.info(f"[DIAG] Cache MISS for {table_name} — querying BQ")
            t0 = time.monotonic()
            contents = table_fetcher.fetch_table_contents(table_name)
            elapsed = time.monotonic() - t0
            fetch_times[table_name] = elapsed
            log.info(
                f"[DIAG] BQ fetch for {table_name} took {elapsed:.2f}s "
                f"({len(contents.records)} rows, {len(contents.fields)} cols)"
            )
            table_cache[table_name] = (contents.fields, contents.records)
            return table_cache[table_name]

        # [DIAG] Log iteration order
        ordered_keys = list(context.selected_asset_keys)
        log.info(
            f"[DIAG] Iteration order ({len(ordered_keys)} assets):"
        )
        for i, ak in enumerate(ordered_keys):
            rj, fc = asset_key_to_job_config[ak]
            log.info(
                f"[DIAG]   #{i+1:3d} {ak.to_user_string()} "
                f"table={rj.source_table_name} fmt={fc.file_extension}"
            )

        # [DIAG] Collect unique tables and log them
        unique_tables = set()
        for ak in ordered_keys:
            rj, _ = asset_key_to_job_config[ak]
            unique_tables.add(rj.source_table_name)
        log.info(
            f"[DIAG] Unique source tables: {len(unique_tables)} "
            f"for {len(ordered_keys)} assets"
        )

        selected_keys = [
            key.to_user_string() for key in context.selected_asset_keys
        ]

        parent_ctx, batch_attrs = get_asset_span_context_and_attrs(
            context, "file_report_assets"
        )
        batch_attrs["file_report.asset_count"] = str(len(selected_keys))
        batch_attrs["file_report.assets"] = str(selected_keys)

        completed_keys: List[str] = []
        batch_t0 = time.monotonic()

        with tracer.start_as_current_span(
            "file_report_batch", context=parent_ctx
        ) as batch_span:
            for key, value in batch_attrs.items():
                batch_span.set_attribute(key, value)

            for idx, asset_key in enumerate(ordered_keys):
                report_job, file_config = asset_key_to_job_config[asset_key]
                asset_key_str = asset_key.to_user_string()
                table_name = report_job.source_table_name

                log.info(
                    f"[DIAG] === Asset #{idx+1}/{len(ordered_keys)}: "
                    f"{asset_key_str} (table={table_name}) ==="
                )

                with tracer.start_as_current_span(
                    f"file_report_{asset_key_str}"
                ) as asset_span:
                    asset_span.set_attribute("asset.name", asset_key_str)
                    asset_span.set_attribute("report.norm", report_job.norm)
                    asset_span.set_attribute(
                        "report.format", file_config.file_extension
                    )

                    try:
                        asset_t0 = time.monotonic()
                        result = generate_single_report(
                            fetch_table=fetch_table_cached,
                            upload_file=file_reports_bucket.upload_file,
                            table_name=report_job.source_table_name,
                            norm=report_job.norm,
                            friendly_name=report_job.friendly_name,
                            file_output_config=file_config,
                            run_id=context.run_id,
                            log=context.log.info,
                        )
                        asset_elapsed = time.monotonic() - asset_t0
                        log.info(
                            f"[DIAG] Asset {asset_key_str} completed in "
                            f"{asset_elapsed:.2f}s"
                        )
                    except Exception as e:
                        asset_elapsed = time.monotonic() - asset_t0
                        batch_elapsed = time.monotonic() - batch_t0
                        log.error(
                            f"[DIAG] !!! FAILURE at asset #{idx+1} "
                            f"{asset_key_str} after {asset_elapsed:.2f}s"
                        )
                        log.error(
                            f"[DIAG] !!! Error type: {type(e).__name__}: {e}"
                        )
                        log.error(
                            f"[DIAG] !!! Table: {table_name}, "
                            f"Format: {file_config.file_extension}"
                        )
                        log.error(
                            f"[DIAG] !!! Cache state: "
                            f"{sorted(table_cache.keys())}"
                        )
                        log.error(
                            f"[DIAG] !!! Was table in cache? "
                            f"{table_name in table_cache}"
                        )
                        log.error(
                            f"[DIAG] !!! Completed {len(completed_keys)}/"
                            f"{len(ordered_keys)} assets in "
                            f"{batch_elapsed:.2f}s before failure"
                        )
                        remaining = [
                            ak.to_user_string()
                            for ak in ordered_keys[idx + 1 :]
                        ]
                        log.error(
                            f"[DIAG] !!! Remaining {len(remaining)} assets "
                            f"that will NOT run: {remaining}"
                        )
                        log.error(
                            f"[DIAG] !!! Full traceback:\n"
                            f"{traceback.format_exc()}"
                        )
                        raise

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

                    completed_keys.append(asset_key_str)
                    yield dg.MaterializeResult(
                        asset_key=asset_key,
                        metadata={
                            "report": dg.MetadataValue.json(report),
                        },
                    )

            # [DIAG] Summary
            batch_elapsed = time.monotonic() - batch_t0
            log.info(
                f"[DIAG] === BATCH COMPLETE: {len(completed_keys)}/"
                f"{len(ordered_keys)} assets in {batch_elapsed:.2f}s ==="
            )
            log.info(
                f"[DIAG] BQ fetches: {len(fetch_times)} unique tables, "
                f"total fetch time: {sum(fetch_times.values()):.2f}s"
            )
            for tbl, t in sorted(fetch_times.items(), key=lambda x: -x[1]):
                log.info(f"[DIAG]   {tbl}: {t:.2f}s")

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
