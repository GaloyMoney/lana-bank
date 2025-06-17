import os
from datetime import datetime
from google.cloud import bigquery, storage
from dicttoxml import dicttoxml
from google.oauth2 import service_account


def main():
    # Read configuration from environment
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

    # Initialize BigQuery client with credentials
    bq_client = bigquery.Client(project=project_id, credentials=credentials)

    # List all tables in the dataset and process those starting with "report_"
    tables_iter = bq_client.list_tables(dataset)
    for table in tables_iter:
        if not table.table_id.startswith("report_"):
            continue
        report_name = table.table_id
        query = f"SELECT * FROM `{project_id}.{dataset}.{report_name}`;"
        query_job = bq_client.query(query)
        rows = query_job.result()
        print(query, query_job.schema); exit()
        field_names = [field.name for field in rows.schema]
        rows_data = [{name: row[name] for name in field_names} for row in rows]
        xml_bytes = dicttoxml(
            rows_data, custom_root="rows", item_root="row", attr_type=False
        )
        xml_content = xml_bytes.decode("utf-8")
        date_str = datetime.now().strftime("%Y-%m-%d")
        blob_path = f"reports/{date_str}/{report_name}.xml"
        storage_client = storage.Client(project=project_id, credentials=credentials)
        bucket = storage_client.bucket(bucket_name)
        blob = bucket.blob(blob_path)
        blob.upload_from_string(xml_content, content_type="text/xml")
        print(f"Uploaded XML report to gs://{bucket_name}/{blob_path}")


# If this script is run as __main__, execute main()
if __name__ == "__main__":
    main()
