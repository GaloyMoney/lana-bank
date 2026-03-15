import dagster as dg

RESOURCE_KEY_SUMSUB = "sumsub"


class SumsubResource(dg.ConfigurableResource):
    """Dagster resource for Sumsub API credentials."""

    def get_key(self) -> str:
        key = dg.EnvVar("LANA_DOMAIN_CONFIG_SUMSUB_API_KEY").get_value()
        if not key:
            raise ValueError(
                "LANA_DOMAIN_CONFIG_SUMSUB_API_KEY environment variable is not set or empty. "
                "Please configure it in your environment."
            )
        return key

    def get_secret(self) -> str:
        secret = dg.EnvVar("LANA_DOMAIN_CONFIG_SUMSUB_API_SECRET").get_value()
        if not secret:
            raise ValueError(
                "LANA_DOMAIN_CONFIG_SUMSUB_API_SECRET environment variable is not set or empty. "
                "Please configure it in your environment."
            )
        return secret

    def get_auth(self) -> tuple[str, str]:
        return self.get_key(), self.get_secret()
