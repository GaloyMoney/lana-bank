import csv
import io
import xml.etree.ElementTree as ET
from datetime import datetime
from typing import Dict

import dagster as dg
from src.core import Protoasset
from src.resources import RESOURCE_KEY_GCS, GCSResource


def _generate_sample_data() -> list[dict]:
    """Generate sample data for reports."""
    return [
        {"id": "1", "name": "Sample Item 1", "value": "100.50", "date": "2025-11-24"},
        {"id": "2", "name": "Sample Item 2", "value": "250.75", "date": "2025-11-24"},
        {"id": "3", "name": "Sample Item 3", "value": "175.25", "date": "2025-11-24"},
    ]


def _generate_csv_report() -> bytes:
    """Generate a CSV report."""
    data = _generate_sample_data()
    output = io.StringIO()
    
    if data:
        writer = csv.DictWriter(output, fieldnames=data[0].keys())
        writer.writeheader()
        writer.writerows(data)
    
    return output.getvalue().encode('utf-8')


def _generate_xml_report() -> bytes:
    """Generate an XML report."""
    data = _generate_sample_data()
    
    root = ET.Element("report")
    root.set("generated_at", datetime.utcnow().isoformat())
    
    items = ET.SubElement(root, "items")
    for item in data:
        item_elem = ET.SubElement(items, "item")
        for key, value in item.items():
            child = ET.SubElement(item_elem, key)
            child.text = str(value)
    
    return ET.tostring(root, encoding='utf-8', xml_declaration=True)


def _generate_txt_report() -> bytes:
    """Generate a TXT report."""
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
    
    return "\n".join(lines).encode('utf-8')


def report_sample_csv(
    context: dg.AssetExecutionContext, gcs: GCSResource
) -> Dict[str, str]:
    """Generate and upload a CSV report to GCS."""
    context.log.info("Generating CSV report...")
    
    # Generate the report
    report_content = _generate_csv_report()
    
    # Upload to GCS
    timestamp = datetime.utcnow().strftime("%Y%m%d_%H%M%S")
    file_path = f"reports/csv/sample_report_{timestamp}.csv"
    gcs_path = gcs.upload_file(
        content=report_content,
        path=file_path,
        content_type="text/csv"
    )
    
    context.log.info(f"CSV report uploaded to: {gcs_path}")
    
    # Add metadata to the materialization
    context.add_output_metadata(
        {
            "gcs_path": gcs_path,
            "file_size_bytes": len(report_content),
            "timestamp": timestamp,
            "format": "csv",
        }
    )
    
    return {
        "gcs_path": gcs_path,
        "format": "csv",
        "timestamp": timestamp,
    }


def report_sample_xml(
    context: dg.AssetExecutionContext, gcs: GCSResource
) -> Dict[str, str]:
    """Generate and upload an XML report to GCS."""
    context.log.info("Generating XML report...")
    
    # Generate the report
    report_content = _generate_xml_report()
    
    # Upload to GCS
    timestamp = datetime.utcnow().strftime("%Y%m%d_%H%M%S")
    file_path = f"reports/xml/sample_report_{timestamp}.xml"
    gcs_path = gcs.upload_file(
        content=report_content,
        path=file_path,
        content_type="application/xml"
    )
    
    context.log.info(f"XML report uploaded to: {gcs_path}")
    
    # Add metadata to the materialization
    context.add_output_metadata(
        {
            "gcs_path": gcs_path,
            "file_size_bytes": len(report_content),
            "timestamp": timestamp,
            "format": "xml",
        }
    )
    
    return {
        "gcs_path": gcs_path,
        "format": "xml",
        "timestamp": timestamp,
    }


def report_sample_txt(
    context: dg.AssetExecutionContext, gcs: GCSResource
) -> Dict[str, str]:
    """Generate and upload a TXT report to GCS."""
    context.log.info("Generating TXT report...")
    
    # Generate the report
    report_content = _generate_txt_report()
    
    # Upload to GCS
    timestamp = datetime.utcnow().strftime("%Y%m%d_%H%M%S")
    file_path = f"reports/txt/sample_report_{timestamp}.txt"
    gcs_path = gcs.upload_file(
        content=report_content,
        path=file_path,
        content_type="text/plain"
    )
    
    context.log.info(f"TXT report uploaded to: {gcs_path}")
    
    # Add metadata to the materialization
    context.add_output_metadata(
        {
            "gcs_path": gcs_path,
            "file_size_bytes": len(report_content),
            "timestamp": timestamp,
            "format": "txt",
        }
    )
    
    return {
        "gcs_path": gcs_path,
        "format": "txt",
        "timestamp": timestamp,
    }


def inform_lana_of_reports(
    context: dg.AssetExecutionContext,
    report_sample_csv: Dict[str, str],
    report_sample_xml: Dict[str, str],
    report_sample_txt: Dict[str, str],
) -> None:
    """
    Inform Lana system of all generated reports.
    This asset runs after all report assets have completed.
    """
    context.log.info("Informing Lana of generated reports...")
    
    # Collect all report information
    reports = [
        report_sample_csv,
        report_sample_xml,
        report_sample_txt,
    ]
    
    # Log detailed information about each report
    for report in reports:
        context.log.info(
            f"Report generated: format={report['format']}, "
            f"gcs_path={report['gcs_path']}, "
            f"timestamp={report['timestamp']}"
        )
    
    # In a real implementation, you would:
    # 1. Send this information to the Lana system via API call
    # 2. Store notification status in a database
    
    context.log.info("All reports have been generated and uploaded successfully.")
    context.log.info("Notification would be sent to Lana system here.")
    
    # Add metadata about the notification
    context.add_output_metadata(
        {
            "notification_time": datetime.utcnow().isoformat(),
            "dependent_reports": [r["format"] for r in reports],
            "report_paths": [r["gcs_path"] for r in reports],
            "status": "completed",
        }
    )


def file_report_protoassets() -> Dict[str, Protoasset]:
    """Return all file report protoassets keyed by asset key."""
    return {
        "report_sample_csv": Protoasset(
            key="report_sample_csv",
            callable=report_sample_csv,
            required_resource_keys={RESOURCE_KEY_GCS},
            tags={"report_type": "csv", "category": "sample"},
        ),
        "report_sample_xml": Protoasset(
            key="report_sample_xml",
            callable=report_sample_xml,
            required_resource_keys={RESOURCE_KEY_GCS},
            tags={"report_type": "xml", "category": "sample"},
        ),
        "report_sample_txt": Protoasset(
            key="report_sample_txt",
            callable=report_sample_txt,
            required_resource_keys={RESOURCE_KEY_GCS},
            tags={"report_type": "txt", "category": "sample"},
        ),
        "inform_lana_of_reports": Protoasset(
            key="inform_lana_of_reports",
            callable=inform_lana_of_reports,
            ins={
                "report_sample_csv": "report_sample_csv",
                "report_sample_xml": "report_sample_xml",
                "report_sample_txt": "report_sample_txt",
            },
            tags={"category": "notification"},
        ),
    }

