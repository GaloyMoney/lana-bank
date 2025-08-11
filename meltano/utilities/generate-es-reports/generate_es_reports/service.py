import os
from pathlib import Path

import yaml

from generate_es_reports.logging import SingletonLogger
from generate_es_reports.constants import Constants
from generate_es_reports.domain.report import (
    CSVFileOutputConfig,
    ReportJobDefinition,
    TXTFileOutputConfig,
    XMLFileOutputConfig,
)
from generate_es_reports.io import (
    BaseReportStorer,
    BaseTableFetcher,
    BigQueryTableFetcher,
    GCSReportStorer,
    LocalReportStorer,
    XMLSchemaRepository,
)
from google.oauth2 import service_account

logger = SingletonLogger().get_logger()


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


def get_report_storer(config: "ReportGeneratorConfig") -> BaseReportStorer:
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


def get_table_fetcher(config: "ReportGeneratorConfig") -> BaseTableFetcher:

    table_fetcher = BigQueryTableFetcher(
        keyfile_path=config.keyfile,
        project_id=config.project_id,
        dataset=config.dataset,
    )

    return table_fetcher


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


class ReportBatch:

    def __init__(self, config: ReportGeneratorConfig):
        self.run_id = config.run_id
        reports_config_yaml_path = Path(__file__).resolve().parent / "reports.yml"
        self.report_jobs = load_report_jobs_from_yaml(reports_config_yaml_path)
        self.table_fetcher = get_table_fetcher(config=config)
        self.report_storer = get_report_storer(config=config)

    def generate_batch(self):
        for report_job in self.report_jobs:
            logger.info(f"Working on report: {report_job.norm}-{report_job.id}")
            table_contents = self.table_fetcher.fetch_table_contents(
                report_job.source_table_name
            )

            for file_output_config in report_job.file_output_configs:
                logger.info(f"Storing as {file_output_config.file_extension}.")
                storable_report = file_output_config.rows_to_report_output(
                    table_contents=table_contents
                )
                path_without_extension = f"reports/{self.run_id}/{report_job.norm}/{report_job.friendly_name}"
                full_path = (
                    path_without_extension + "." + file_output_config.file_extension
                )
                self.report_storer.store_report(path=full_path, report=storable_report)

            logger.info(f"Finished: {report_job.norm}-{report_job.id}")


def run_report_batch():
    logger.info("Starting run.")

    report_generator_config = get_config_from_env()
    report_batch = ReportBatch(config=report_generator_config)
    report_batch.generate_batch()

    logger.info("Finished run.")
