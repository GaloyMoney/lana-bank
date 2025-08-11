from __future__ import annotations
from pathlib import Path
from xml.etree import ElementTree
import logging, logging.config
from abc import ABC, abstractmethod
import io
import os
import csv

from generate_es_reports.domain.report import (
    StorableReportOutput,
    TabularReportContents,
)
from google.cloud import bigquery, storage
from google.oauth2 import service_account
from xmlschema import XMLSchema

from generate_es_reports.constants import Constants
from generate_es_reports.logging import SingletonLogger

logger = SingletonLogger().get_logger()


class BaseFileOutputConfig(ABC):

    file_extension: str = NotImplemented
    content_type: str = NotImplemented

    def __init_subclass__(cls):

        mandatory_class_attributes = ("file_extension", "content_type")

        for attribute in mandatory_class_attributes:
            if getattr(cls, attribute) is NotImplemented:
                raise NotImplementedError(f"{cls.__name__} must define '{attribute}'")

    @abstractmethod
    def rows_to_report_output(
        self, table_contents: TabularReportContents
    ) -> StorableReportOutput:
        pass


class XMLFileOutputConfig(BaseFileOutputConfig):

    file_extension = "xml"
    content_type = "text/xml"

    def __init__(self, xml_schema: Union[XMLSchema, None] = None) -> None:
        self.xml_schema = xml_schema
        self.target_namespace = self.xml_schema.target_namespace
        self.root_element_tag = next(iter(self.xml_schema.elements), None)
        self.sequence_elements_tag = self._extract_sequence_elements_tag()

    def rows_to_report_output(
        self, table_contents: TabularReportContents
    ) -> StorableReportOutput:
        field_names = table_contents.fields
        rows_data = table_contents.records

        xml_root_element = ElementTree.Element(
            f"{{{self.target_namespace}}}" + f"{self.root_element_tag}"
        )

        for row in rows_data:
            sequence_level_element = ElementTree.SubElement(
                xml_root_element,
                f"{{{self.target_namespace}}}" + f"{self.sequence_elements_tag}",
            )
            for field, value in row.items():

                new_field_element = ElementTree.SubElement(
                    sequence_level_element,
                    f"{{{self.target_namespace}}}" + f"{field}",
                )
                new_field_element.text = value

        xml_string = ElementTree.tostring(xml_root_element, encoding="unicode")

        output = io.StringIO()
        output.write(xml_string)
        report_content = output.getvalue()

        report_has_content = len(rows_data) > 0
        is_xml_valid = self.xml_schema.is_valid(source=report_content)
        if report_has_content and not is_xml_valid:
            logger.warning(f"Schema validation for report failed. Listing errors.")
            for err in self.xml_schema.iter_errors(report_content):
                logger.debug(f"Path: {err.path}, Reason: {err.reason}")
                logger.debug(f"  Source: {err.source}")

        return StorableReportOutput(
            report_content=report_content, report_content_type=self.content_type
        )

    def _extract_sequence_elements_tag(self) -> str:
        """Extract the tag of the sequence elements of the schema.

        This makes a strong assumption that the XSD follows the common
        structure of SSF reports: one root element followed by a sequence
        of children elements, all within the same namespace.

        This will 100% break on XSD that follow other patterns.

        Returns:
            str: the tag for the sequence elements of this XSD.
        """
        elem = self.xml_schema.elements[self.root_element_tag]
        model = elem.type.content

        first_child = next(model.iter_elements(), None)

        # Strip namespace if present
        qname = first_child.name
        child_name = qname.split("}", 1)[-1] if qname.startswith("{") else qname

        return child_name


class CSVFileOutputConfig(BaseFileOutputConfig):

    file_extension = "csv"
    content_type = "text/plain"

    def __init__(self, delimiter: str = ",", lineterminator: str = "\n") -> None:
        self.delimiter = delimiter
        self.lineterminator = lineterminator

    def rows_to_report_output(
        self, table_contents: TabularReportContents
    ) -> StorableReportOutput:
        field_names = table_contents.fields
        rows_data = table_contents.records

        output = io.StringIO()

        writer = csv.DictWriter(
            output,
            fieldnames=field_names,
            delimiter=self.delimiter,
            lineterminator=self.lineterminator,
        )
        writer.writeheader()
        writer.writerows(rows_data)
        report_content = output.getvalue()

        return StorableReportOutput(
            report_content=report_content, report_content_type=self.content_type
        )


class TXTFileOutputConfig(BaseFileOutputConfig):

    file_extension = "txt"
    content_type = "text/plain"

    def __init__(self, delimiter: str = "|", lineterminator: str = "\n") -> None:
        self.delimiter = delimiter
        self.lineterminator = lineterminator

    def rows_to_report_output(
        self, table_contents: TabularReportContents
    ) -> StorableReportOutput:
        field_names = table_contents.fields
        rows_data = table_contents.records

        output = io.StringIO()

        writer = csv.DictWriter(
            output,
            fieldnames=field_names,
            delimiter=self.delimiter,
            lineterminator=self.lineterminator,
        )
        writer.writeheader()
        writer.writerows(rows_data)
        report_content = output.getvalue()

        return StorableReportOutput(
            report_content=report_content, report_content_type=self.content_type
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
