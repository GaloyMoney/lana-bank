import os
import io
import csv
from pathlib import Path
import logging, logging.config
from abc import ABC, abstractmethod
from typing import Union

import yaml
from google.cloud import bigquery, storage
from google.oauth2 import service_account
from xmlschema import XMLSchema
from xml.etree import ElementTree

logging.basicConfig(
    level=logging.DEBUG,
    format="%(asctime)s [%(levelname)s] - %(message)s",
    handlers=[logging.StreamHandler()],
)

# Disable logging by external packages to prevent them from
# bloating the console output
logging.config.dictConfig(
    {
        "version": 1,
        "disable_existing_loggers": True,
    }
)

logger = logging.getLogger(name="generate-es-reports")


class Constants:
    """Simple namespace to store constants and avoid magic vars."""

    DBT_BIGQUERY_PROJECT_ENVVAR_KEY = "DBT_BIGQUERY_PROJECT"
    DBT_BIGQUERY_DATASET_ENVVAR_KEY = "DBT_BIGQUERY_DATASET"
    DOCS_BUCKET_NAME_ENVVAR_KEY = "DOCS_BUCKET_NAME"
    GOOGLE_APPLICATION_CREDENTIALS_ENVVAR_KEY = "GOOGLE_APPLICATION_CREDENTIALS"
    AIRFLOW_CTX_DAG_RUN_ID_ENVVAR_KEY = "AIRFLOW_CTX_DAG_RUN_ID"
    USE_LOCAL_FS_ENVVAR_KEY = "USE_LOCAL_FS"
    NAME_PREFIX_ENVVAR_KEY = "NAME_PREFIX"

    DEFAULT_XML_SCHEMAS_PATH = Path(__file__).resolve().parent / "schemas"


class StorableReportOutput:
    """The contents of a report file, together with their content type."""

    def __init__(self, report_content_type: str, report_content: str) -> None:
        self.content_type = report_content_type
        self.content = report_content


class BaseFileOutputConfig(ABC):

    file_extension: str = NotImplemented
    content_type: str = NotImplemented

    def __init_subclass__(cls):

        mandatory_class_attributes = ("file_extension", "content_type")

        for attribute in mandatory_class_attributes:
            if getattr(cls, attribute) is NotImplemented:
                raise NotImplementedError(f"{cls.__name__} must define '{attribute}'")

    @abstractmethod
    def rows_to_report_output(self, rows) -> StorableReportOutput:
        pass


class XMLFileOutputConfig(BaseFileOutputConfig):

    file_extension = "xml"
    content_type = "text/xml"

    def __init__(self, xml_schema: Union[XMLSchema, None] = None) -> None:
        self.xml_schema = xml_schema
        self.target_namespace = self.xml_schema.target_namespace
        self.root_element_tag = next(iter(self.xml_schema.elements), None)
        self.sequence_elements_tag = self._extract_sequence_elements_tag()

    def rows_to_report_output(self, rows) -> StorableReportOutput:
        field_names = [field.name for field in rows.schema]
        rows_data = [{name: row[name] for name in field_names} for row in rows]

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

    def rows_to_report_output(self, rows) -> StorableReportOutput:
        field_names = [field.name for field in rows.schema]
        rows_data = [{name: row[name] for name in field_names} for row in rows]

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

    def rows_to_report_output(self, rows) -> StorableReportOutput:
        field_names = [field.name for field in rows.schema]
        rows_data = [{name: row[name] for name in field_names} for row in rows]

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


class ReportJobDefinition:
    """
    Defines a report that must be fetched and converted into
    certain file formats.
    """

    def __init__(
        self,
        norm: str,
        id: str,
        friendly_name: str,
        file_output_configs: tuple[BaseFileOutputConfig, ...],
    ):
        self.norm = norm
        self.id = id
        self.friendly_name = friendly_name
        self.file_output_configs = file_output_configs

    @property
    def source_table_name(self) -> str:
        return f"report_{self.norm}_{self.id}"


def load_report_jobs_from_yaml(yaml_path: Path) -> tuple[ReportJobDefinition, ...]:
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

    xml_schema_repository = XMLSchemaRepository()

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


class ReportGeneratorConfig:
    """
    The config for one execution of this script.
    """

    def __init__(
        self,
        project_id: str,
        dataset: str,
        bucket_name: str,
        run_id: str,
        keyfile: Path,
        use_gcs: bool,
        use_local_fs: bool,
        name_prefix: str,
    ):
        self.project_id = project_id
        self.dataset = dataset
        self.bucket_name = bucket_name
        self.run_id = run_id
        self.keyfile = keyfile
        self.use_gcs = use_gcs
        self.use_local_fs = use_local_fs
        self.name_prefix = name_prefix


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
        Constants.NAME_PREFIX_ENVVAR_KEY,
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
        name_prefix=os.getenv(Constants.NAME_PREFIX_ENVVAR_KEY),
    )


class ReportStorer(ABC):
    """Abstract interface for an object that can store a report contents as a file somewhere."""

    @abstractmethod
    def store_report(self, path: str, report: StorableReportOutput) -> None:
        """Store a report given a path and contents.

        Args:
            path (str): where to store the report.
            report (StorableReport): a storable report specifying contents and their types.
        """
        pass


class GCSReportStorer(ReportStorer):
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


class LocalReportStorer(ReportStorer):
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


def get_report_storer(config: ReportGeneratorConfig) -> ReportStorer:
    """Infer from the given config what is the right storer to use and set it up.

    Args:
        config (ReportGeneratorConfig): the specific config for this run.

    Raises:
        ValueError: if the config is inconsistent and doesn't make it clear which storer should be used.

    Returns:
        ReportStorer: a concrete, ready to use storer instance for this run.
    """

    if config.use_local_fs:
        return LocalReportStorer()

    if config.use_gcs:
        credentials = service_account.Credentials.from_service_account_file(
            config.keyfile
        )
        return GCSReportStorer(
            gcp_project_id=config.project_id,
            gcp_credentials=credentials,
            target_bucket_name=config.bucket_name,
        )

    raise ValueError("Inconsistent config, can't figure out where to write reports to.")


def main():
    logger.info("Starting run.")
    report_generator_config = get_config_from_env()

    credentials = service_account.Credentials.from_service_account_file(
        report_generator_config.keyfile
    )
    bq_client = bigquery.Client(
        project=report_generator_config.project_id, credentials=credentials
    )

    report_storer: ReportStorer = get_report_storer(config=report_generator_config)

    report_config_yaml_path = Path(__file__).resolve().parent / "reports.yml"
    report_jobs = load_report_jobs_from_yaml(report_config_yaml_path)

    def get_rows_from_table(table_name: str):
        query = f"SELECT * FROM `{report_generator_config.project_id}.{report_generator_config.dataset}.{table_name}`;"
        query_job = bq_client.query(query)
        rows = query_job.result()

        return rows

    for report_job in report_jobs:
        logger.info(f"Working on report: {report_job.norm}-{report_job.id}")
        path_without_extension = f"{report_generator_config.name_prefix}/reports/{report_generator_config.run_id}/{report_job.norm}/{report_job.friendly_name}"

        for file_output_config in report_job.file_output_configs:
            logger.info(f"Storing as {file_output_config.file_extension}.")
            storable_report = file_output_config.rows_to_report_output(
                rows=get_rows_from_table(table_name=report_job.source_table_name)
            )
            full_path = path_without_extension + "." + file_output_config.file_extension
            report_storer.store_report(path=full_path, report=storable_report)

        logger.info(f"Finished: {report_job.norm}-{report_job.id}")

    logger.info("Finished run.")


if __name__ == "__main__":
    main()
