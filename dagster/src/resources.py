from typing import Any

import dagster as dg


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


def get_lana_resources():
    resources = {}
    resources["lana_core_pg"] = PostgresResource()
    resources["dw_bq"] = BigQueryResource()
    return resources
