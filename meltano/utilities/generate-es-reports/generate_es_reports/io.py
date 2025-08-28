from __future__ import annotations
from pathlib import Path
from abc import ABC, abstractmethod
import os
import base64
import re

import yaml
from google.cloud import bigquery, storage
from google.oauth2 import service_account
from xmlschema import XMLSchema

from generate_es_reports.constants import Constants
from generate_es_reports.logging import SingletonLogger
from generate_es_reports.domain.report import (
    CSVFileOutputConfig,
    ReportJobDefinition,
    TXTFileOutputConfig,
    TabularReportContents,
    ReportGeneratorConfig,
    XMLFileOutputConfig,
)

logger = SingletonLogger().get_logger()


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


class GCSReportStorer(BaseReportStorer):
    """A report storer that writes report files to a GCS bucket."""

    def __init__(
        self,
        gcp_project_id: str,
        gcp_credentials: service_account.Credentials,
        target_bucket_name: str,
    ) -> None:
        self._storage_client = storage.Client(
            project=gcp_project_id, credentials=gcp_credentials
        )
        self._bucket = self._storage_client.bucket(bucket_name=target_bucket_name)

    def store_report(self, path: str, report: StorableReportOutput) -> None:
        # GCS blob paths with timestamps in folder names cause problems
        m = re.match(r"^(reports/(manual|scheduled)__)((.+?))(/.+)$", path)
        if m:
            prefix, ts, rest = m.group(1, 3, 5)
            ts_encoded = base64.urlsafe_b64encode(ts.encode()).decode().rstrip("=")
            path = f"{prefix}{ts_encoded}{rest}"

        blob = self._bucket.blob(path)
        logger.info(f"Uploading to {path}...")
        blob.upload_from_string(report.content, content_type=report.content_type)
        logger.info(f"Uploaded")


class LocalReportStorer(BaseReportStorer):
    """A report store that writes into the local filesystem."""

    def __init__(self, root_path: Path = Path("./report_files/")) -> None:
        self._root_path = root_path

    def store_report(self, path: str, report: StorableReportOutput) -> None:
        target_path = self._root_path / path

        os.makedirs(os.path.dirname(target_path), exist_ok=True)
        logger.info(f"Storing locally at: {path}")
        with open(target_path, "w", encoding="utf-8") as f:
            f.write(report.content)
        logger.info("File stored")


class BaseTableFetcher(ABC):
    """
    An interface to somewhere we can read tabular data from to get records for
    a report.
    """

    @abstractmethod
    def fetch_table_contents(self, table_name: str) -> TabularReportContents:
        """Get the table contents somehow and return them in a stable object."

        Returns:
            TabularReportContents: the table contents and the fields listed.
        """
        pass


class BigQueryTableFetcher(BaseTableFetcher):
    """
    Fetches records from a specified Biquery project. It naively gets all the
    contents of the specified tables: all fields, all records. Will definitely
    not be adequate for large tables.
    """

    def __init__(self, keyfile_path: Path, project_id: str, dataset: str):

        self.project_id = project_id
        self.dataset = dataset

        credentials = service_account.Credentials.from_service_account_file(
            keyfile_path
        )

        self._bq_client = bigquery.Client(
            project=self.project_id, credentials=credentials
        )

    def fetch_table_contents(self, table_name: str) -> TabularReportContents:
        query = f"SELECT * FROM `{self.project_id}.{self.dataset}.{table_name}`;"
        query_job = self._bq_client.query(query)
        rows = query_job.result()

        field_names = [field.name for field in rows.schema]
        records = [{name: row[name] for name in field_names} for row in rows]

        table_contents = TabularReportContents(field_names=field_names, records=records)

        return table_contents


class MockTable:

    def __init__(self, name: str, records: tuple[dict]):
        self.name = name
        self.records = records


class MockTableFetcher(BaseTableFetcher):
    """
    Mock implementation for testing purposes. Make an instance, then pass
    mock tables so you can fetch them later by name.
    """

    def __init__(self):
        self.mock_tables = {}

    def add_mock_table(self, mock_table: MockTable):
        self.mock_tables[mock_table.name] = mock_table

    def fetch_table_contents(self, table_name: str) -> TabularReportContents:
        mock_table = self.mock_tables[table_name]

        # We use the keys of the first record, assuming all records share the
        # same keys.
        field_names = mock_table.records[0].keys()
        records = mock_table.records

        table_contents = TabularReportContents(field_names=field_names, records=records)

        return table_contents


def get_config_from_env() -> ReportGeneratorConfig:
    """Read env vars, check that config is consistent and return it.

    Raises:
        RuntimeError: If a required env var is missing.
        FileNotFoundError: If the GCP credentials file can't be found.

    Returns:
        ReportGeneratorConfig: a specific config instance for this run.
    """
    required_envs = [
        Constants.DBT_BIGQUERY_PROJECT_ENVVAR_KEY,
        Constants.DBT_BIGQUERY_DATASET_ENVVAR_KEY,
        Constants.DOCS_BUCKET_NAME_ENVVAR_KEY,
        Constants.GOOGLE_APPLICATION_CREDENTIALS_ENVVAR_KEY,
    ]
    missing = [var for var in required_envs if not os.getenv(var)]
    if missing:
        raise RuntimeError(
            f"Missing required environment variables: {', '.join(missing)}"
        )

    run_id = os.getenv(
        Constants.AIRFLOW_CTX_DAG_RUN_ID_ENVVAR_KEY, "dev"
    )  # If no AIRFLOW, we assume dev env

    keyfile = Path(os.getenv(Constants.GOOGLE_APPLICATION_CREDENTIALS_ENVVAR_KEY))
    if not keyfile.is_file():
        raise FileNotFoundError(
            f"Can't read GCP credentials at: {str(keyfile.absolute())}"
        )

    use_local_fs = bool(os.getenv(Constants.USE_LOCAL_FS_ENVVAR_KEY))

    use_gcs = True
    if use_local_fs:
        use_gcs = False

    return ReportGeneratorConfig(
        project_id=os.getenv(Constants.DBT_BIGQUERY_PROJECT_ENVVAR_KEY),
        dataset=os.getenv(Constants.DBT_BIGQUERY_DATASET_ENVVAR_KEY),
        bucket_name=os.getenv(Constants.DOCS_BUCKET_NAME_ENVVAR_KEY),
        run_id=run_id,
        keyfile=keyfile,
        use_gcs=use_gcs,
        use_local_fs=use_local_fs,
    )


def load_report_jobs_from_yaml(
    yaml_path: Path, xml_schema_repository: XMLSchemaRepository = XMLSchemaRepository()
) -> tuple[ReportJobDefinition, ...]:
    """Read report jobs to do from a YAML file.

    Args:
        yaml_path (Path): path to the YAML that holds the config.

    Returns:
        tuple[ReportJobDefinition, ...]: All the report jobs that must be run.
    """
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
