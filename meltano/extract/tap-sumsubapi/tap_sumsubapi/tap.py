"""SumsubApi tap class."""

from __future__ import annotations

from singer_sdk import Tap
from singer_sdk import typing as th  # JSON schema typing helpers

from tap_sumsubapi.streams import ApplicantStream
import os

STREAM_TYPES = [ApplicantStream]


class TapSumsubApi(Tap):
    """SumsubApi tap class."""

    name = "tap-sumsubapi"

    config_jsonschema = th.PropertiesList(
        th.Property(
            "host",
            th.StringType,
            description=(
                "Hostname for postgres instance. "
                + "Note if sqlalchemy_url is set this will be ignored."
            ),
        ),
        th.Property(
            "port",
            th.IntegerType,
            default=5432,
            description=(
                "The port on which postgres is awaiting connection. "
                + "Note if sqlalchemy_url is set this will be ignored."
            ),
        ),
        th.Property(
            "user",
            th.StringType,
            description=(
                "User name used to authenticate. "
                + "Note if sqlalchemy_url is set this will be ignored."
            ),
        ),
        th.Property(
            "password",
            th.StringType,
            secret=True,
            description=(
                "Password used to authenticate. "
                "Note if sqlalchemy_url is set this will be ignored."
            ),
        ),
        th.Property(
            "database",
            th.StringType,
            description=(
                "Database name. "
                + "Note if sqlalchemy_url is set this will be ignored."
            ),
        ),
        th.Property(
            "secret",
            th.StringType,
            description="Example: Hej2ch71kG2kTd1iIUDZFNsO5C1lh5Gq",
        ),
        th.Property(
            "key",
            th.StringType,
            description="Example: sbx:uY0CgwELmgUAEyl4hNWxLngb.0WSeQeiYny4WEqmAALEAiK2qTC96fBad",
        ),
    ).to_dict()

    def discover_streams(self):
        """Return a list of discovered streams."""
        return [stream_class(tap=self) for stream_class in STREAM_TYPES]

    @property
    def postgres_host(self):
        """Get Postgres host from config or environment."""
        return os.getenv('TAP_POSTGRES_HOST') or self.config.get('host')

    @property
    def postgres_port(self):
        """Get Postgres port from config or environment."""
        return os.getenv('TAP_POSTGRES_PORT') or self.config.get('port', 5432)

    @property
    def postgres_user(self):
        """Get Postgres user from config or environment."""
        return os.getenv('TAP_POSTGRES_USER') or self.config.get('user')

    @property
    def postgres_password(self):
        """Get Postgres password from config or environment."""
        return os.getenv('TAP_POSTGRES_PASSWORD') or self.config.get('password')

    @property
    def postgres_database(self):
        """Get Postgres database from config or environment."""
        return os.getenv('TAP_POSTGRES_DATABASE') or self.config.get('database')

    @property
    def sumsub_key(self):
        """Get Sumsub key from config or environment."""
        return os.getenv('SUMSUB_KEY') or self.config.get('key')

    @property
    def sumsub_secret(self):
        """Get Sumsub secret from config or environment."""
        return os.getenv('SUMSUB_SECRET') or self.config.get('secret')

if __name__ == "__main__":
    TapSumsubApi.cli()
