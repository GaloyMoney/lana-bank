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


class StorableReportOutput:
    """The contents of a report file, together with their format."""

    def __init__(self, report_content_type: str, report_content: str) -> None:
        self.content_type = report_content_type
        self.content = report_content


class BaseFileOutputConfig(ABC):

    file_extension = NotImplemented
    content_type = NotImplemented

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

    def __init__(self) -> None:
        pass

    def rows_to_report_output(self, rows) -> StorableReportOutput:
        field_names = [field.name for field in rows.schema]
        rows_data = [{name: row[name] for name in field_names} for row in rows]

        xml_string = dicttoxml(rows_data, custom_root="rows", attr_type=False).decode(
            "utf-8"
        )
        output = io.StringIO()
        output.write(xml_string)
        report_content = output.getvalue()

        return StorableReportOutput(
            report_content=report_content, report_content_type=self.content_type
        )


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


class ReportJobDefinition:

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
    def table_name(self) -> str:
        return f"report_{self.norm}_{self.id}"


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

    gcs_report_storer = report_storer

    report_jobs = (
        # NRP_41 Reports - XML & CSV
        # ReportJobDefinition(
        #     norm=Constants.NRP_41_ID,
        #     id="01_persona",
        #     friendly_name="persona",
        #     file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        # ),
        ReportJobDefinition(
            norm=Constants.NRP_41_ID,
            id="02_referencia",
            friendly_name="referencia",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        ReportJobDefinition(
            norm=Constants.NRP_41_ID,
            id="03_referencia_garantia",
            friendly_name="referencia_garantia",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        ReportJobDefinition(
            norm=Constants.NRP_41_ID,
            id="04_garantia_hipotecaria",
            friendly_name="garantia_hipotecaria",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        ReportJobDefinition(
            norm=Constants.NRP_41_ID,
            id="05_garantia_fiduciaria",
            friendly_name="garantia_fiduciaria",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        ReportJobDefinition(
            norm=Constants.NRP_41_ID,
            id="06_garantia_aval",
            friendly_name="garantia_aval",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        ReportJobDefinition(
            norm=Constants.NRP_41_ID,
            id="07_garantia_pignorada",
            friendly_name="garantia_pignorada",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        ReportJobDefinition(
            norm=Constants.NRP_41_ID,
            id="08_garantia_prenda",
            friendly_name="garantia_prenda",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        ReportJobDefinition(
            norm=Constants.NRP_41_ID,
            id="09_garantia_bono",
            friendly_name="garantia_bono",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        ReportJobDefinition(
            norm=Constants.NRP_41_ID,
            id="10_garantia_poliza",
            friendly_name="garantia_poliza",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        ReportJobDefinition(
            norm=Constants.NRP_41_ID,
            id="11_garantia_fondo",
            friendly_name="garantia_fondo",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        ReportJobDefinition(
            norm=Constants.NRP_41_ID,
            id="12_referencia_gasto",
            friendly_name="referencia_gasto",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        ReportJobDefinition(
            norm=Constants.NRP_41_ID,
            id="13_referencia_unidad",
            friendly_name="referencia_unidad",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        ReportJobDefinition(
            norm=Constants.NRP_41_ID,
            id="14_referencia_cancelada",
            friendly_name="referencia_cancelada",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        ReportJobDefinition(
            norm=Constants.NRP_41_ID,
            id="15_socios_sociedades",
            friendly_name="socios_sociedades",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        ReportJobDefinition(
            norm=Constants.NRP_41_ID,
            id="16_junta_directiva",
            friendly_name="junta_directiva",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        ReportJobDefinition(
            norm=Constants.NRP_41_ID,
            id="17_garantia_prendaria",
            friendly_name="garantia_prendaria",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        # NRP_51 Reports - XML & CSV
        ReportJobDefinition(
            norm=Constants.NRP_51_ID,
            id="01_saldo_cuenta",
            friendly_name="saldo_cuenta",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        ReportJobDefinition(
            norm=Constants.NRP_51_ID,
            id="02_deposito_extranjero",
            friendly_name="deposito_extranjero",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        ReportJobDefinition(
            norm=Constants.NRP_51_ID,
            id="03_dato_extracontable",
            friendly_name="dato_extracontable",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        ReportJobDefinition(
            norm=Constants.NRP_51_ID,
            id="04_titulo_valor_extranjero",
            friendly_name="titulo_valor_extranjero",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        ReportJobDefinition(
            norm=Constants.NRP_51_ID,
            id="05_prestamo_garantizado",
            friendly_name="prestamo_garantizado",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        ReportJobDefinition(
            norm=Constants.NRP_51_ID,
            id="06_aval_garantizado",
            friendly_name="aval_garantizado",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        ReportJobDefinition(
            norm=Constants.NRP_51_ID,
            id="07_deuda_subordinada",
            friendly_name="deuda_subordinada",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        ReportJobDefinition(
            norm=Constants.NRP_51_ID,
            id="08_balance_proyectado",
            friendly_name="balance_proyectado",
            file_output_configs=(XMLFileOutputConfig(), CSVFileOutputConfig()),
        ),
        # NRSF_03 Reports - XML & TXT
        # ReportJobDefinition(
        #     norm=Constants.NRSF_03_ID,
        #     id="01_cliente",
        #     friendly_name="cliente",
        #     file_output_configs=(XMLFileOutputConfig(), TXTFileOutputConfig()),
        # ),
        # ReportJobDefinition(
        #     norm=Constants.NRSF_03_ID,
        #     id="02_depósitos",
        #     friendly_name="depósitos",
        #     file_output_configs=(XMLFileOutputConfig(), TXTFileOutputConfig()),
        # ),
        # ReportJobDefinition(
        #     norm=Constants.NRSF_03_ID,
        #     id="03_documentos_clientes",
        #     friendly_name="documentos_clientes",
        #     file_output_configs=(XMLFileOutputConfig(), TXTFileOutputConfig()),
        # ),
        ReportJobDefinition(
            norm=Constants.NRSF_03_ID,
            id="04_titulares",
            friendly_name="titulares",
            file_output_configs=(XMLFileOutputConfig(), TXTFileOutputConfig()),
        ),
        # ReportJobDefinition(
        #     norm=Constants.NRSF_03_ID,
        #     id="05_agencias",
        #     friendly_name="agencias",
        #     file_output_configs=(XMLFileOutputConfig(), TXTFileOutputConfig()),
        # ),
        ReportJobDefinition(
            norm=Constants.NRSF_03_ID,
            id="06_productos",
            friendly_name="productos",
            file_output_configs=(XMLFileOutputConfig(), TXTFileOutputConfig()),
        ),
        # ReportJobDefinition(
        #     norm=Constants.NRSF_03_ID,
        #     id="07_funcionarios_y_empleados",
        #     friendly_name="funcionarios_y_empleados",
        #     file_output_configs=(XMLFileOutputConfig(), TXTFileOutputConfig()),
        # ),
        # ReportJobDefinition(
        #     norm=Constants.NRSF_03_ID,
        #     id="08_resumen_de_depósitos_garantizados",
        #     friendly_name="resumen_de_depósitos_garantizados",
        #     file_output_configs=(XMLFileOutputConfig(), TXTFileOutputConfig()),
        # ),
        ReportJobDefinition(
            norm=Constants.NRSF_03_ID,
            id="09_ajustes",
            friendly_name="ajustes",
            file_output_configs=(XMLFileOutputConfig(), TXTFileOutputConfig()),
        ),
    )

    def get_rows_from_table(table_name: str):
        query = f"SELECT * FROM `{report_generator_config.project_id}.{report_generator_config.dataset}.{table_name}`;"
        query_job = bq_client.query(query)
        rows = query_job.result()

        return rows

    for report_job in report_jobs:
        logger.info(f"Working on report: {report_job.norm}-{report_job.id}")
        path_without_extension = f"reports/{report_generator_config.run_id}/{report_job.norm}/{report_job.friendly_name}"

        for file_output_config in report_job.file_output_configs:
            logger.info(f"Storing as {file_output_config.file_extension}.")
            storable_report = file_output_config.rows_to_report_output(
                rows=get_rows_from_table(table_name=report_job.table_name)
            )
            full_path = path_without_extension + "." + file_output_config.file_extension
            report_storer.store_report(path=full_path, report=storable_report)

        logger.info(f"Finished: {report_job.norm}-{report_job.id}")

    logger.info("Finished run.")


if __name__ == "__main__":
    main()
