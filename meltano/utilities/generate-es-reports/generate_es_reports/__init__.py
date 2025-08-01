import os
import io
import csv
from re import compile
from pathlib import Path
import logging, logging.config
from abc import ABC, abstractmethod

from google.cloud import bigquery, storage
from dicttoxml import dicttoxml
from google.oauth2 import service_account

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] - %(message)s",
    handlers=[logging.StreamHandler()],
)

# Disable logging by external packages
logging.config.dictConfig(
    {
        "version": 1,
        "disable_existing_loggers": True,
    }
)

logger = logging.getLogger(name="generate-es-reports")


class Constants:
    """Simple namespace to store constants and avoid magic vars."""

    TABLE_NAME_PATTERN = compile(r"report_([0-9a-z_]+)_\d+_(.+)")

    DBT_BIGQUERY_PROJECT_ENVVAR_KEY = "DBT_BIGQUERY_PROJECT"
    DBT_BIGQUERY_DATASET_ENVVAR_KEY = "DBT_BIGQUERY_DATASET"
    DOCS_BUCKET_NAME_ENVVAR_KEY = "DOCS_BUCKET_NAME"
    GOOGLE_APPLICATION_CREDENTIALS_ENVVAR_KEY = "GOOGLE_APPLICATION_CREDENTIALS"
    AIRFLOW_CTX_DAG_RUN_ID_ENVVAR_KEY = "AIRFLOW_CTX_DAG_RUN_ID"
    USE_LOCAL_FS_ENVVAR_KEY = "USE_LOCAL_FS"

    NRP_41_ID = "nrp_41"
    NRP_51_ID = "nrp_51"
    NRSF_03_ID = "nrsf_03"

    XML_FORMATTABLE_NORMS = (NRP_41_ID, NRP_51_ID)
    TXT_FORMATTABLE_NORMS = (NRSF_03_ID,)
    CSV_FORMATTABLE_NORMS = (NRP_41_ID, NRP_51_ID, NRSF_03_ID)


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
    ):
        self.project_id = project_id
        self.dataset = dataset
        self.bucket_name = bucket_name
        self.run_id = run_id
        self.keyfile = keyfile
        self.use_gcs = use_gcs
        self.use_local_fs = use_local_fs


class FileOutputConfig(ABC):

    file_extension = NotImplemented
    content_type = NotImplemented

    def __init_subclass__(cls):

        mandatory_class_attributes = ("file_extension", "content_type")

        for attribute in mandatory_class_attributes:
            if getattr(cls, attribute) is NotImplemented:
                raise NotImplementedError(f"{cls.__name__} must define '{attribute}'")


class XMLFileOutputConfig(FileOutputConfig):

    file_extension = "xml"
    content_type = "text/xml"

    def __init__(self) -> None:
        pass


class CSVFileOutputConfig(FileOutputConfig):

    file_extension = "csv"
    content_type = "text/plain"

    def __init__(self, delimiter: str = ",", lineterminator: str = "\n") -> None:
        self.delimiter = delimiter
        self.lineterminator = lineterminator


class TXTFileOutputConfig(FileOutputConfig):

    file_extension = "txt"
    content_type = "text/plain"

    def __init__(self, delimiter: str = "|", lineterminator: str = "\n") -> None:
        self.delimiter = delimiter
        self.lineterminator = lineterminator


class ReportJobDefinition:

    def __init__(
        self, norm: str, id: str, file_output_configs: tuple[FileOutputConfig, ...]
    ):
        self.norm = norm
        self.id = id
        self.file_output_configs = file_output_configs


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


class StorableReport:
    """The contents of a report file, together with their format."""

    def __init__(self, report_content_type: str, report_content: str) -> None:
        self.content_type = report_content_type
        self.content = report_content


class ReportStorer(ABC):
    """Abstract interface for an object that can store a report contents as a file somewhere."""

    @abstractmethod
    def store_report(self, path: str, report: StorableReport) -> None:
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

    def store_report(self, path: str, report: StorableReport) -> None:
        blob = self._bucket.blob(path)
        logger.info(f"Uploading to {path}...")
        blob.upload_from_string(report.content, content_type=report.content_type)
        logger.info(f"Uploaded")


class LocalReportStorer(ReportStorer):
    """A report store that writes into the local filesystem."""

    def __init__(self, root_path: Path = Path("./report_files/")) -> None:
        self._root_path = root_path

    def store_report(self, path: str, report: StorableReport) -> None:
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

    gcs_report_storer = report_storer

    tables_iter = bq_client.list_tables(report_generator_config.dataset)

    report_jobs = (
        ReportJobDefinition(
            norm=Constants.NRP_41_ID,
            id="06_garantia_aval",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        ReportJobDefinition(
            norm=Constants.NRP_51_ID,
            id="06_aval_garantizado",
            file_output_configs=(XMLFileOutputConfig(), TXTFileOutputConfig()),
        ),
        ReportJobDefinition(
            norm=Constants.NRSF_03_ID,
            id="06_productos",
            file_output_configs=(TXTFileOutputConfig(), CSVFileOutputConfig()),
        ),
    )

    def get_rows_from_table(table_name: str):
        query = f"SELECT * FROM `{report_generator_config.project_id}.{report_generator_config.dataset}.{table_name}`;"
        query_job = bq_client.query(query)
        rows = query_job.result()

        return rows

    # for report_job in report_jobs:
    for table in tables_iter:
        table_name = table.table_id
        match = Constants.TABLE_NAME_PATTERN.match(table_name)
        if not match:
            continue
        logger.info(f"Working on table {table_name}.")
        norm_name = match.group(1)
        report_name = match.group(2)

        rows = get_rows_from_table(table_name=table_name)
        field_names = [field.name for field in rows.schema]
        rows_data = [{name: row[name] for name in field_names} for row in rows]

        blob_path = (
            f"reports/{report_generator_config.run_id}/{norm_name}/{report_name}"
        )

        if norm_name in Constants.XML_FORMATTABLE_NORMS:
            xml_string = dicttoxml(
                rows_data, custom_root="rows", attr_type=False
            ).decode("utf-8")
            output = io.StringIO()
            output.write(xml_string)
            report_content = output.getvalue()
            full_blob_path = blob_path + ".xml"
            gcs_report_storer.store_report(
                path=full_blob_path,
                report=StorableReport(
                    report_content=report_content,
                    report_content_type="text/xml",
                ),
            )

        if norm_name == Constants.TXT_FORMATTABLE_NORMS:
            output = io.StringIO()
            writer = csv.DictWriter(
                output, fieldnames=field_names, delimiter="|", lineterminator="\n"
            )
            writer.writeheader()
            writer.writerows(rows_data)
            report_content = output.getvalue()
            full_blob_path = blob_path + ".txt"
            gcs_report_storer.store_report(
                path=full_blob_path,
                report=StorableReport(
                    report_content=report_content,
                    report_content_type="text/plain",
                ),
            )

        # CSV versions of all regulatory reports
        if norm_name in Constants.CSV_FORMATTABLE_NORMS:
            output = io.StringIO()
            writer = csv.DictWriter(
                output, fieldnames=field_names, delimiter=",", lineterminator="\n"
            )
            writer.writeheader()
            writer.writerows(rows_data)
            report_content = output.getvalue()
            full_blob_path = blob_path + ".csv"
            gcs_report_storer.store_report(
                path=full_blob_path,
                report=StorableReport(
                    report_content=report_content,
                    report_content_type="text/plain",
                ),
            )

    logger.info("Finished run.")


if __name__ == "__main__":
    main()
