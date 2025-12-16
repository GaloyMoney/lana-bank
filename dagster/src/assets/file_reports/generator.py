"""Dagster wrapper for report generation - adapts resources to generate_es_reports."""

from typing import Any

import dagster as dg

from generate_es_reports.domain.report import BaseFileOutputConfig
from generate_es_reports.generator import generate_single_report

from src.resources.bigquery import BigQueryResource
from src.resources.gcs import GCSResource


def generate_report_file(
    context: dg.AssetExecutionContext,
    bq_resource: BigQueryResource,
    gcs_resource: GCSResource,
    table_name: str,
    norm: str,
    friendly_name: str,
    file_output_config: BaseFileOutputConfig,
    run_id: str,
) -> dict[str, Any]:
    """Generate a single report file using dagster resources.

    Thin wrapper that adapts dagster resources to the generate_es_reports API.
    """
    return generate_single_report(
        fetch_table=bq_resource.fetch_table,
        upload_file=gcs_resource.upload_file,
        table_name=table_name,
        norm=norm,
        friendly_name=friendly_name,
        file_output_config=file_output_config,
        run_id=run_id,
        log=context.log.info,
    )
