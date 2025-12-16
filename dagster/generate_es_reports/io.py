from __future__ import annotations
from pathlib import Path
from abc import ABC, abstractmethod
import base64
import re

import yaml
from xmlschema import XMLSchema

from generate_es_reports.constants import Constants
from generate_es_reports.domain.report import (
    CSVFileOutputConfig,
    ReportJobDefinition,
    TXTFileOutputConfig,
    XMLFileOutputConfig,
    StorableReportOutput,
)


class XMLSchemaRepository:
    """Provides access to the xsd schemas in the schemas folder."""

    xml_schema_extension = ".xsd"

    def __init__(self, schema_folder_path: Path = Constants.DEFAULT_XML_SCHEMAS_PATH):
        self.schema_folder_path = schema_folder_path

    def get_schema(self, schema_id: str) -> XMLSchema:
        full_schema_file_path = self.schema_folder_path / (
            schema_id + self.xml_schema_extension
        )
        return XMLSchema(full_schema_file_path)


class BaseReportStorer(ABC):
    """Abstract interface for an object that can store a report contents as a file somewhere."""

    @abstractmethod
    def store_report(self, path: str, report: StorableReportOutput) -> None:
        """Store a report given a path and contents.

        Args:
            path (str): where to store the report.
            report (StorableReport): a storable report specifying contents and their types.
        """
        pass


def encode_gcs_path(path: str) -> str:
    """Encode timestamps in GCS paths to avoid issues with special characters.
    
    GCS blob paths with timestamps in folder names cause problems.
    """
    m = re.match(r"^(reports/(manual|scheduled)__)((.+?))(/.+)$", path)
    if m:
        prefix, ts, rest = m.group(1, 3, 5)
        ts_encoded = base64.urlsafe_b64encode(ts.encode()).decode().rstrip("=")
        path = f"{prefix}{ts_encoded}{rest}"
    return path


def load_report_jobs_from_yaml(
    yaml_path: Path, xml_schema_repository: XMLSchemaRepository = None
) -> tuple[ReportJobDefinition, ...]:
    """Read report jobs to do from a YAML file.

    Args:
        yaml_path (Path): path to the YAML that holds the config.
        xml_schema_repository: Optional schema repository. If None, uses default.

    Returns:
        tuple[ReportJobDefinition, ...]: All the report jobs that must be run.
    """
    if xml_schema_repository is None:
        xml_schema_repository = XMLSchemaRepository()

    with open(yaml_path, "r", encoding="utf-8") as file:
        data = yaml.safe_load(file)

    str_to_type_mapping = {
        "xml": XMLFileOutputConfig,
        "csv": CSVFileOutputConfig,
        "txt": TXTFileOutputConfig,
    }

    report_jobs = []
    for report_job in data["report_jobs"]:
        output_configs = []
        for output in report_job["outputs"]:
            if output["type"] == "xml":
                output_config = XMLFileOutputConfig(
                    xml_schema=xml_schema_repository.get_schema(
                        schema_id=output["validation_schema_id"]
                    )
                )
                output_configs.append(output_config)
                continue

            output_config = str_to_type_mapping[output["type"].lower()]()
            output_configs.append(output_config)

        output_configs = tuple(output_configs)

        report_jobs.append(
            ReportJobDefinition(
                norm=report_job["norm"],
                id=report_job["id"],
                friendly_name=report_job["friendly_name"],
                file_output_configs=output_configs,
            )
        )

    return tuple(report_jobs)
