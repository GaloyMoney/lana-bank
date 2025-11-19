import dagster as dg

from src.dlt_sources.resources import (
    BigQueryResource,
    PostgresResource,
)

def get_lana_resources():
    resources = {}
    resources["lana_core_pg"] = PostgresResource()
    resources["dw_bq"] = BigQueryResource(
        base64_credentials=dg.EnvVar("TF_VAR_sa_creds").get_value(),
        target_dataset=dg.EnvVar("TARGET_BIGQUERY_DATASET").get_value(),
    )
    return resources