from __future__ import annotations
from pathlib import Path
from abc import ABC, abstractmethod
import base64
import re

import yaml
from google.cloud import bigquery
from google.oauth2 import service_account
from xmlschema import XMLSchema

from generate_es_reports.constants import DEFAULT_XML_SCHEMAS_PATH, DEFAULT_REPORTS_YAML_PATH
from generate_es_reports.domain.report import (
    CSVFileOutputConfig,
    ReportJobDefinition,
    TabularReportContents,
    TXTFileOutputConfig,
    XMLFileOutputConfig,
    StorableReportOutput,
)


class XMLSchemaRepository:
    """Provides access to the xsd schemas in the schemas folder."""

    xml_schema_extension = ".xsd"

    def __init__(self, schema_folder_path: Path = DEFAULT_XML_SCHEMAS_PATH):
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


class BaseTableFetcher(ABC):
    """Interface for fetching tabular data from a data source."""

    @abstractmethod
    def fetch_table_contents(self, table_name: str) -> TabularReportContents:
        """Fetch table contents and return them as TabularReportContents.

        Args:
            table_name: Name of the table to fetch.

        Returns:
            TabularReportContents with field names and records.
        """
        pass


class BigQueryTableFetcher(BaseTableFetcher):
    """Fetches records from a BigQuery dataset.

    Naively gets all contents of specified tables: all fields, all records.
    Not suitable for very large tables.
    """

    def __init__(self, credentials_dict: dict, dataset: str):
        """Initialize BigQuery table fetcher.

        Args:
            credentials_dict: Service account credentials as a dictionary.
            dataset: BigQuery dataset name.
        """
        self.dataset = dataset
        self.project_id = credentials_dict["project_id"]

        credentials = service_account.Credentials.from_service_account_info(
            credentials_dict
        )
        self._bq_client = bigquery.Client(
            project=self.project_id, credentials=credentials
        )

    def fetch_table_contents(self, table_name: str) -> TabularReportContents:
        """Fetch all rows from a BigQuery table.

        Args:
            table_name: Name of the table (without dataset prefix).

        Returns:
            TabularReportContents with field names and records.
        """
        query = f"SELECT * FROM `{self.project_id}.{self.dataset}.{table_name}`;"
        query_job = self._bq_client.query(query)
        rows = query_job.result()

        field_names = tuple(field.name for field in rows.schema)
        records = [{name: row[name] for name in field_names} for row in rows]

        return TabularReportContents(field_names=field_names, records=records)


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


def load_default_report_jobs() -> tuple[ReportJobDefinition, ...]:
    """Load report jobs from the package's default reports.yml with default schemas."""
    return load_report_jobs_from_yaml(DEFAULT_REPORTS_YAML_PATH)
