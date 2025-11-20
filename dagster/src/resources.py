from typing import Any

import dagster as dg


RESOURCE_KEY_LANA_CORE_PG = "lana_core_pg"
RESOURCE_KEY_DW_BQ = "dw_bq"


class PostgresResource(dg.ConfigurableResource):
    """Dagster resource for PostgreSQL connection configuration."""

    def get_connection_string(self) -> str:
        return dg.EnvVar("LANA_PG_CON").get_value()


class BigQueryResource(dg.ConfigurableResource):
    """Dagster resource for BigQuery configuration."""

    def get_base64_credentials(self) -> str:
        return dg.EnvVar("TF_VAR_sa_creds").get_value()

    def get_target_dataset(self) -> str:
        return dg.EnvVar("TARGET_BIGQUERY_DATASET").get_value()


def get_project_resources():
    resources = {}
    resources[RESOURCE_KEY_LANA_CORE_PG] = PostgresResource()
    resources[RESOURCE_KEY_DW_BQ] = BigQueryResource()
    return resources
