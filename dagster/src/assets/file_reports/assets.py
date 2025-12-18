"""Factory for creating file report dagster assets from reports.yml configuration."""

from typing import Dict, List

import dagster as dg

from generate_es_reports.io import load_default_report_jobs
from generate_es_reports.generator import generate_single_report
from generate_es_reports.domain.report import BaseFileOutputConfig

from src.core import Protoasset
from src.resources.bigquery import RESOURCE_KEY_DW_BQ
from src.resources.gcs import RESOURCE_KEY_FILE_REPORTS_BUCKET


def _make_asset_key(norm: str, report_id: str, file_type: str) -> str:
    """Generate asset key name: report_{norm}_{id}_{format}"""
    return f"report_{norm}_{report_id}_{file_type}"


def _create_report_callable(
    table_name: str,
    norm: str,
    friendly_name: str,
    file_output_config: BaseFileOutputConfig,
):
    """Create a callable for a specific report asset."""

    def report_asset_fn(context: dg.AssetExecutionContext, dw_bq, file_reports_bucket):
        result = generate_single_report(
            fetch_table=dw_bq.fetch_table,
            upload_file=file_reports_bucket.upload_file,
            table_name=table_name,
            norm=norm,
            friendly_name=friendly_name,
            file_output_config=file_output_config,
            run_id=context.run_id,
            log=context.log.info,
        )

        context.add_output_metadata({
            "gcs_path": result["gcs_path"],
            "row_count": result["row_count"],
            "norm": norm,
            "file_type": result["file_type"],
        })

    return report_asset_fn


def file_report_protoassets() -> Dict[str, Protoasset]:
    """Create protoassets for all report files defined in reports.yml.

    Creates one asset per norm + report + file format combination.
    Asset naming: report_{norm}_{id}_{format}

    Returns:
        Dict mapping asset key names to Protoasset objects.
    """
    report_jobs = load_default_report_jobs()

    protoassets = {}

    for job in report_jobs:
        for output_config in job.file_output_configs:
            asset_key_name = _make_asset_key(
                norm=job.norm,
                report_id=job.id,
                file_type=output_config.file_extension,
            )

            callable_fn = _create_report_callable(
                table_name=job.source_table_name,
                norm=job.norm,
                friendly_name=job.friendly_name,
                file_output_config=output_config,
            )

            # TODO: Add dbt model dependencies once they exist
            # deps = [dg.AssetKey(["dbt_lana_dw", "reports", job.source_table_name])]

            protoassets[asset_key_name] = Protoasset(
                key=dg.AssetKey(asset_key_name),
                callable=callable_fn,
                required_resource_keys={RESOURCE_KEY_DW_BQ, RESOURCE_KEY_FILE_REPORTS_BUCKET},
                tags={
                    "category": "file_report",
                    "norm": job.norm,
                    "report_id": job.id,
                    "file_type": output_config.file_extension,
                },
            )

    return protoassets


def get_all_report_asset_keys() -> List[dg.AssetKey]:
    """Get list of all report asset keys for job creation."""
    protoassets = file_report_protoassets()
    return [p.key for p in protoassets.values()]
