import os
from datetime import datetime
from google.cloud import bigquery, storage
from dicttoxml import dicttoxml

def main():
    # Read configuration from environment
    required_envs = ["TARGET_BIGQUERY_PROJECT", "TARGET_BIGQUERY_DATASET", "TARGET_BIGQUERY_TABLE", "DOCS_BUCKET"]
    missing = [var for var in required_envs if not os.getenv(var)]
    if missing:
        raise RuntimeError(f"Missing required environment variables: {', '.join(missing)}")
    project_id = os.getenv("TARGET_BIGQUERY_PROJECT")
    dataset = os.getenv("TARGET_BIGQUERY_DATASET")
    table = os.getenv("TARGET_BIGQUERY_TABLE")
    bucket_name = os.getenv("DOCS_BUCKET")
    report_name = os.getenv("REPORT_NAME", "report")  # default to "report" if not provided

    # Initialize BigQuery client (credentials via environment) and run query
    bq_client = bigquery.Client(project=project_id)  # project optional if credentials provide a default
    query = f"SELECT * FROM `{project_id}.{dataset}.{table}`;"
    query_job = bq_client.query(query)
    rows = query_job.result()  # Wait for query to complete and get an iterator of rows
    
    # Convert query results to a list of dicts for XML conversion
    field_names = [field.name for field in query_job.schema]  # get column names from job schema
    rows_data = [{name: row[name] for name in field_names} for row in rows]

    # Convert to XML string with custom root "<rows>" and without type attributes
    xml_bytes = dicttoxml(rows_data, custom_root='rows', item_root='row', attr_type=False)
    xml_content = xml_bytes.decode('utf-8')

    # Determine file path in GCS: reports/YYYY-MM-DD/report_name.xml
    date_str = datetime.now().strftime("%Y-%m-%d")
    blob_path = f"reports/{date_str}/{report_name}.xml"

    # Upload the XML report to GCS
    storage_client = storage.Client(project=project_id)  # uses env credentials
    bucket = storage_client.bucket(bucket_name)
    blob = bucket.blob(blob_path)
    blob.upload_from_string(xml_content, content_type="text/xml")

    print(f"Uploaded XML report to gs://{bucket_name}/{blob_path}")

# If this script is run as __main__, execute main()
if __name__ == "__main__":
    main()
