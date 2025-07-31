import os
import io
import csv
from abc import ABC, abstractmethod
from datetime import datetime
from google.cloud import bigquery, storage
from dicttoxml import dicttoxml
from google.oauth2 import service_account
from re import compile
from pathlib import Path
from typing import Any

from .validation import Validator


table_name_pattern = compile(r"report_([0-9a-z_]+)_\d+_(.+)")


class ReportGeneratorConfig:

    def __init__(
        self,
        project_id: str,
        dataset: str,
        bucket_name: str,
        run_id: str,
        keyfile: Path,
    ):
        self.project_id = project_id
        self.dataset = dataset
        self.bucket_name = bucket_name
        self.run_id = run_id
        self.keyfile = keyfile


def get_config_from_env() -> ReportGeneratorConfig:

    required_envs = [
        "DBT_BIGQUERY_PROJECT",
        "DBT_BIGQUERY_DATASET",
        "DOCS_BUCKET_NAME",
        "GOOGLE_APPLICATION_CREDENTIALS",
    ]
    missing = [var for var in required_envs if not os.getenv(var)]
    if missing:
        raise RuntimeError(
            f"Missing required environment variables: {', '.join(missing)}"
        )

    run_id = os.getenv(
        "AIRFLOW_CTX_DAG_RUN_ID", "dev"
    )  # If no AIRFLOW, we assume dev env

    keyfile = Path(os.getenv("GOOGLE_APPLICATION_CREDENTIALS"))
    if not keyfile.is_file():
        raise FileNotFoundError(
            f"Can't read GCP credentials at: {str(keyfile.absolute)}"
        )

    return ReportGeneratorConfig(
        project_id=os.getenv("DBT_BIGQUERY_PROJECT"),
        dataset=os.getenv("DBT_BIGQUERY_DATASET"),
        bucket_name=os.getenv("DOCS_BUCKET_NAME"),
        run_id=run_id,
        keyfile=keyfile,
    )


class StorableReport:

    def __init__(self, report_type: Any, report_content: Any) -> None:
        self.type = report_type
        self.content = report_content


class ReportStorer:

    @abstractmethod
    def store_report(self, path: str, report: StorableReport) -> None:
        pass


class GCSReportStorer:

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
        print(f"Uploading to {path}...")
        blob.upload_from_string(report.content, content_type=report.type)
        print(f"Uploaded")


def main():
    report_generator_config = get_config_from_env()

    credentials = service_account.Credentials.from_service_account_file(
        report_generator_config.keyfile
    )
    bq_client = bigquery.Client(
        project=report_generator_config.project_id, credentials=credentials
    )

    gcs_report_storer = GCSReportStorer(
        gcp_project_id=report_generator_config.project_id,
        gcp_credentials=credentials,
        target_bucket_name=report_generator_config.bucket_name,
    )

    tables_iter = bq_client.list_tables(report_generator_config.dataset)

    for table in tables_iter:
        table_name = table.table_id
        match = table_name_pattern.match(table_name)
        if not match:
            continue
        norm_name = match.group(1)
        report_name = match.group(2)

        query = f"SELECT * FROM `{report_generator_config.project_id}.{report_generator_config.dataset}.{table_name}`;"
        query_job = bq_client.query(query)
        rows = query_job.result()
        field_names = [field.name for field in rows.schema]
        rows_data = [{name: row[name] for name in field_names} for row in rows]

        blob_path = (
            f"reports/{report_generator_config.run_id}/{norm_name}/{report_name}"
        )

        if norm_name in ["nrp_41", "nrp_51"]:
            report_content_type = "text/xml"
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
                    report_content=report_content, report_type=report_content_type
                ),
            )

        if norm_name == "nrsf_03":
            report_content_type = "text/plain"
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
                    report_content=report_content, report_type=report_content_type
                ),
            )

        # CSV versions of all regulatory reports
        if norm_name in ["nrp_41", "nrp_51", "nrsf_03"]:
            report_content_type = "text/plain"
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
                    report_content=report_content, report_type=report_content_type
                ),
            )


if __name__ == "__main__":
    main()
