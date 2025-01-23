"""BitfinexApi tap class."""

from __future__ import annotations

from singer_sdk import Tap
from singer_sdk import typing as th  # JSON schema typing helpers

from tap_bitfinexapi import TickerStream

STREAM_TYPES = [TickerStream]

class TapBitfinexApi(Tap):
    """BitfinexApi tap class."""

    name = "tap-bitfinexapi"

    # TODO: Update this section with the actual config values you expect:
    config_jsonschema = th.PropertiesList(
        th.Property(
            "api_url",
            th.StringType,
            title="API URL",
            default="https://api.mysample.com",
            description="The url for the API service",
        ),
        th.Property(
            "user_agent",
            th.StringType,
            description=(
                "A custom User-Agent header to send with each request. Default is "
                "'<tap_name>/<tap_version>'"
            ),
        ),
    ).to_dict()

    def discover_streams(self) -> list[streams.BitfinexApiStream]:
        """Return a list of discovered streams.
        """
        return [stream_class(tap=self) for stream_class in STREAM_TYPES]


if __name__ == "__main__":
    TapBitfinexApi.cli()
