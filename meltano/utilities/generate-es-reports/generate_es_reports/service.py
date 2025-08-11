from pathlib import Path

from generate_es_reports.domain.report import ReportGeneratorConfig

from generate_es_reports.logging import SingletonLogger
from generate_es_reports.io import (
    BaseReportStorer,
    BaseTableFetcher,
    BigQueryTableFetcher,
    GCSReportStorer,
    LocalReportStorer,
    get_config_from_env,
    load_report_jobs_from_yaml,
)
from google.oauth2 import service_account

logger = SingletonLogger().get_logger()


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
