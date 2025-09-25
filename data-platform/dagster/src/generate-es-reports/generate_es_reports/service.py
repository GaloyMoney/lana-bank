from __future__ import annotations
from pathlib import Path

from generate_es_reports.domain.report import ReportGeneratorConfig, ReportBatch
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


def run_report_batch() -> None:
    """
    e2e tool execution. Get all config from env and files, execute the report
    batch.
    """
    logger.info("Starting run.")

    report_generator_config = get_config_from_env()
    reports_config_yaml_path = Path(__file__).resolve().parent / "reports.yml"
    report_jobs = load_report_jobs_from_yaml(reports_config_yaml_path)
    table_fetcher = get_table_fetcher(config=report_generator_config)
    report_storer = get_report_storer(config=report_generator_config)
    report_batch = ReportBatch(
        run_id=report_generator_config.run_id,
        report_jobs=report_jobs,
        table_fetcher=table_fetcher,
        report_storer=report_storer,
    )
    report_batch.generate_batch()

    logger.info("Finished run.")
