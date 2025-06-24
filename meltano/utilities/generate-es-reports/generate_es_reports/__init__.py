import os
import io
import csv
from datetime import datetime
from google.cloud import bigquery, storage
from dicttoxml import dicttoxml
from google.oauth2 import service_account
from re import compile

table_name_pattern = compile(r"report_([0-9a-z_]+)_\d+_(.+)")


def main():
    required_envs = [
        "DBT_BIGQUERY_PROJECT",
        "DBT_BIGQUERY_DATASET",
        "DOCS_BUCKET_NAME",
    ]
    missing = [var for var in required_envs if not os.getenv(var)]
    if missing:
        raise RuntimeError(
            f"Missing required environment variables: {', '.join(missing)}"
        )
    project_id = os.getenv("DBT_BIGQUERY_PROJECT")
    dataset = os.getenv("DBT_BIGQUERY_DATASET")
    bucket_name = os.getenv("DOCS_BUCKET_NAME")

    keyfile = os.getenv("GOOGLE_APPLICATION_CREDENTIALS")
    if not keyfile or not os.path.isfile(keyfile):
        raise RuntimeError(
            "GOOGLE_APPLICATION_CREDENTIALS environment variable must be set to the path of a valid service account JSON file."
        )
    credentials = service_account.Credentials.from_service_account_file(keyfile)
    bq_client = bigquery.Client(project=project_id, credentials=credentials)

    tables_iter = bq_client.list_tables(dataset)
    for table in tables_iter:
        table_name = table.table_id
        match = table_name_pattern.match(table_name)
        if not match:
            continue
        norm_name = match.group(1)
        report_name = match.group(2)
        date_str = datetime.now().strftime("%Y-%m-%d")

        query = f"SELECT * FROM `{project_id}.{dataset}.{table_name}`;"
        query_job = bq_client.query(query)
        rows = query_job.result()
        field_names = [field.name for field in rows.schema]
        rows_data = [{name: row[name] for name in field_names} for row in rows]

        if norm_name == "nrp_41":
            report_content_type = "text/xml"
            report_bytes = dicttoxml(rows_data, custom_root="rows", attr_type=False)
            report_content = report_bytes.decode("utf-8")
            blob_path = f"reports/{date_str}/{norm_name}/{report_name}.xml"

        elif norm_name == "nrsf_03":
            report_content_type = "text/plain"
            output = io.StringIO()
            writer = csv.DictWriter(
                output, fieldnames=field_names, delimiter="|", lineterminator="\n"
            )
            writer.writeheader()
            writer.writerows(rows_data)
            report_content = output.getvalue()
            blob_path = f"reports/{date_str}/{norm_name}/{report_name}.txt"

        else:
            # TODO: we'll have to come up with some kind of convention for
            # serializing ad-hoc internal reports when we have those
            pass

        storage_client = storage.Client(project=project_id, credentials=credentials)
        bucket = storage_client.bucket(bucket_name)
        blob = bucket.blob(blob_path)
        blob.upload_from_string(report_content, content_type=report_content_type)
        print(f"Uploaded XML report to gs://{bucket_name}/{blob_path}")


if __name__ == "__main__":
    main()
