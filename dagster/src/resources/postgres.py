import dagster as dg

RESOURCE_KEY_LANA_CORE_PG = "lana_core_pg"


class PostgresResource(dg.ConfigurableResource):
    """Dagster resource for PostgreSQL connection configuration."""

    def get_connection_string(self) -> str:
        return dg.EnvVar("LANA_PG_CON").get_value()
