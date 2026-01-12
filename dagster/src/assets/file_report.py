import csv
import inspect
import io
import sys
import xml.etree.ElementTree as ET
from datetime import datetime
from typing import Callable, Dict, List, Literal, Optional, TypedDict

import dagster as dg
from generate_es_reports.domain.report import BaseFileOutputConfig, ReportJobDefinition
from generate_es_reports.generator import generate_single_report
from generate_es_reports.io import BigQueryTableFetcher
from src.assets.dbt import _load_dbt_manifest, _get_dbt_asset_key
from src.core import Protoasset
from src.resources import (
    RESOURCE_KEY_DW_BQ,
    RESOURCE_KEY_FILE_REPORTS_BUCKET,
    BigQueryResource,
    GCSResource,
)


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
    """Create a callable that generates and uploads a single report file.
    """

    def _report_callable(
        context: dg.AssetExecutionContext,
        dw_bq: BigQueryResource,
        file_reports_bucket: GCSResource,
    ) -> None:
        table_fetcher = BigQueryTableFetcher(
            credentials_dict=dw_bq.get_credentials_dict(),
            dataset=dw_bq.get_target_dataset(),
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

        # Add metadata for tracking
        context.add_output_metadata(
            {
                "norm": result["norm"],
                "friendly_name": result["friendly_name"],
                "file_type": result["file_type"],
                "gcs_path": result["gcs_path"],
                "row_count": result["row_count"],
            }
        )

    return _report_callable


class ReportFile(TypedDict):
    """Represents a single file in a report."""

    type: Literal["csv", "txt", "xml"]
    path_in_bucket: str


class Report(TypedDict):
    """Represents a report with its metadata and associated files."""

    name: str
    norm: str
    files: List[ReportFile]


### vvv START - TEMPORARY FUNCTIONS TO BE REPLACED WITH REAL REPORT GENERATION FUNCTIONS - START vvv ###
def _generate_sample_data() -> list[dict]:
    return [
        {"id": "1", "name": "Sample Item 1", "value": "100.50", "date": "2025-11-24"},
        {"id": "2", "name": "Sample Item 2", "value": "250.75", "date": "2025-11-24"},
        {"id": "3", "name": "Sample Item 3", "value": "175.25", "date": "2025-11-24"},
    ]


def _generate_csv_report() -> bytes:
    data = _generate_sample_data()
    output = io.StringIO()

    if data:
        writer = csv.DictWriter(output, fieldnames=data[0].keys())
        writer.writeheader()
        writer.writerows(data)

    return output.getvalue().encode("utf-8")


def _generate_xml_report() -> bytes:
    data = _generate_sample_data()

    root = ET.Element("report")
    root.set("generated_at", datetime.utcnow().isoformat())

    items = ET.SubElement(root, "items")
    for item in data:
        item_elem = ET.SubElement(items, "item")
        for key, value in item.items():
            child = ET.SubElement(item_elem, key)
            child.text = str(value)

    return ET.tostring(root, encoding="utf-8", xml_declaration=True)


def _generate_txt_report() -> bytes:
    data = _generate_sample_data()

    lines = ["Sample Report", "=" * 50, ""]
    lines.append(f"Generated at: {datetime.utcnow().isoformat()}")
    lines.append("")

    for item in data:
        lines.append(f"ID: {item['id']}")
        lines.append(f"  Name: {item['name']}")
        lines.append(f"  Value: {item['value']}")
        lines.append(f"  Date: {item['date']}")
        lines.append("")

    return "\n".join(lines).encode("utf-8")


def report_sample_1(
    context: dg.AssetExecutionContext, file_reports_bucket: GCSResource
) -> None:
    """Generate and upload sample report 1 (CSV format) to GCS."""
    report_content = _generate_csv_report()

    context.log.info("Uploading report 1 (CSV) to GCS...")
    timestamp = datetime.utcnow().strftime("%Y%m%d_%H%M%S")
    file_path = f"reports/sample_1/report_{timestamp}.csv"
    gcs_path = file_reports_bucket.upload_file(
        content=report_content, path=file_path, content_type="text/csv"
    )
    context.log.info(f"Report 1 uploaded to: {gcs_path}")

    # Store report information in metadata using the Report type
    report: Report = {
        "name": "sample_1",
        "norm": "sample_report_norm_1",
        "files": [
            {
                "type": "csv",
                "path_in_bucket": gcs_path,
            }
        ],
    }

    context.add_output_metadata({"reports": [report]})


def report_sample_2(
    context: dg.AssetExecutionContext, file_reports_bucket: GCSResource
) -> None:
    """Generate and upload 3 files for sample report 2 to GCS."""
    context.log.info("Generating 3 files for sample_2...")

    timestamp = datetime.utcnow().strftime("%Y%m%d_%H%M%S")
    files: List[ReportFile] = []

    # File 1: XML format
    context.log.info("Generating XML file...")
    xml_content = _generate_xml_report()
    xml_file_path = f"reports/sample_2/report_{timestamp}_data.xml"
    xml_gcs_path = file_reports_bucket.upload_file(
        content=xml_content, path=xml_file_path, content_type="application/xml"
    )
    context.log.info(f"XML file uploaded to: {xml_gcs_path}")
    files.append(
        {
            "type": "xml",
            "path_in_bucket": xml_gcs_path,
        }
    )

    # File 2: CSV summary
    context.log.info("Generating CSV file...")
    csv_content = _generate_csv_report()
    csv_file_path = f"reports/sample_2/report_{timestamp}_summary.csv"
    csv_gcs_path = file_reports_bucket.upload_file(
        content=csv_content, path=csv_file_path, content_type="text/csv"
    )
    context.log.info(f"CSV file uploaded to: {csv_gcs_path}")
    files.append(
        {
            "type": "csv",
            "path_in_bucket": csv_gcs_path,
        }
    )

    # File 3: TXT details
    context.log.info("Generating TXT file...")
    txt_content = _generate_txt_report()
    txt_file_path = f"reports/sample_2/report_{timestamp}_details.txt"
    txt_gcs_path = file_reports_bucket.upload_file(
        content=txt_content, path=txt_file_path, content_type="text/plain"
    )
    context.log.info(f"TXT file uploaded to: {txt_gcs_path}")
    files.append(
        {
            "type": "txt",
            "path_in_bucket": txt_gcs_path,
        }
    )

    context.log.info(
        f"Successfully generated and uploaded {len(files)} files for sample_2"
    )

    # Store report information in metadata using the Report type
    report: Report = {
        "name": "sample_2",
        "norm": "sample_report_norm_2",
        "files": files,
    }

    context.add_output_metadata({"reports": [report]})


def report_sample_3(
    context: dg.AssetExecutionContext, file_reports_bucket: GCSResource
) -> None:
    """Generate and upload sample report 3 (TXT format) to GCS."""

    context.log.info("Uploading report 3 (TXT) to GCS...")
    report_content = _generate_txt_report()
    timestamp = datetime.utcnow().strftime("%Y%m%d_%H%M%S")
    file_path = f"reports/sample_3/report_{timestamp}.txt"
    gcs_path = file_reports_bucket.upload_file(
        content=report_content, path=file_path, content_type="text/plain"
    )
    context.log.info(f"Report 3 uploaded to: {gcs_path}")

    # Store report information in metadata using the Report type
    report: Report = {
        "name": "sample_3",
        "norm": "sample_report_norm_3",
        "files": [
            {
                "type": "txt",
                "path_in_bucket": gcs_path,
            }
        ],
    }

    context.add_output_metadata({"reports": [report]})


### ^^^ END - TEMPORARY FUNCTIONS TO BE REPLACED WITH REAL REPORT GENERATION FUNCTIONS - END ^^^ ###


def _discover_reports() -> Dict[str, callable]:
    """Fetch all functions that start with report_"""
    current_module = sys.modules[__name__]
    reports = {}

    for name, obj in inspect.getmembers(current_module, inspect.isfunction):
        if name.startswith("report_"):
            reports[name] = obj

    return reports


def _extract_reports_from_asset(
    context: dg.AssetExecutionContext, asset_key_str: str
) -> List[Report]:
    """Extract report metadata from a materialized asset."""
    asset_key = dg.AssetKey(asset_key_str)
    materialization = context.instance.get_latest_materialization_event(asset_key)

    if not (materialization and materialization.asset_materialization):
        return []

    metadata = materialization.asset_materialization.metadata
    if "reports" not in metadata:
        return []

    reports_metadata = metadata["reports"]
    reports_list = getattr(reports_metadata, "value", reports_metadata)

    return reports_list


def inform_lana_of_new_reports(context: dg.AssetExecutionContext) -> None:
    """Collect all generated reports and notify Lana system."""
    all_reports: List[Report] = []
    reports = _discover_reports()

    for asset_key_str in reports.keys():
        all_reports.extend(_extract_reports_from_asset(context, asset_key_str))

    context.log.info(f"Total reports collected: {len(all_reports)}")
    for report in all_reports:
        file_types = [f["type"] for f in report["files"]]
        context.log.info(
            f"Report: name={report['name']}, "
            f"norm={report['norm']}, "
            f"files={len(report['files'])} ({', '.join(file_types)})"
        )

    context.log.info("TODO: Notification would be sent to Lana system here.")


def file_report_protoassets() -> Dict[str, Protoasset]:
    """Create protoassets for all discovered report generation functions."""
    report_protoassets = {}
    reports = _discover_reports()

    for report_name, report_callable in reports.items():
        report_protoassets[report_name] = Protoasset(
            key=dg.AssetKey(report_name),
            callable=report_callable,
            required_resource_keys={RESOURCE_KEY_FILE_REPORTS_BUCKET},
            tags={"category": "report_generation", "report_name": report_name},
        )

    return report_protoassets


def inform_lana_protoasset() -> Protoasset:
    """Create protoasset for informing Lana of new reports."""
    return Protoasset(
        key=dg.AssetKey("inform_lana_of_new_reports"),
        callable=inform_lana_of_new_reports,
        tags={"category": "notification"},
    )
