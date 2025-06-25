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
    def sumsub_key(self):
        """Get Sumsub key from config or environment."""
        return self.config.get('key') or os.getenv('SUMSUB_KEY')

    @property
    def sumsub_secret(self):
        """Get Sumsub secret from config or environment."""
        return self.config.get('secret') or os.getenv('SUMSUB_SECRET')


if __name__ == "__main__":
    TapSumsubApi.cli()
