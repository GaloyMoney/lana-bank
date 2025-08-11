from __future__ import annotations
from pathlib import Path
from abc import ABC, abstractmethod
import os


from google.cloud import bigquery, storage
from google.oauth2 import service_account
from xmlschema import XMLSchema

from generate_es_reports.constants import Constants
from generate_es_reports.logging import SingletonLogger
from generate_es_reports.domain.report import TabularReportContents

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
        pass


class BigQueryTableFetcher(BaseTableFetcher):

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


class MockTableFetcher(BaseTableFetcher):
    pass
