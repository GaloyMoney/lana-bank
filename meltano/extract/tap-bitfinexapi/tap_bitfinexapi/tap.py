"""BitfinexApi tap class."""

from __future__ import annotations

from singer_sdk import Tap
from singer_sdk import typing as th  # JSON schema typing helpers

from tap_bitfinexapi.streams import (
    BitfinexTickerStream,
    BitfinexTradesStream,
    BitfinexOrderBookStream,
)

STREAM_TYPES = [BitfinexTickerStream, BitfinexTradesStream, BitfinexOrderBookStream]


class TapBitfinexApi(Tap):
    """BitfinexApi tap class."""

    name = "tap-bitfinexapi"

    def discover_streams(self):
        """Return a list of discovered streams."""
        return [stream_class(tap=self) for stream_class in STREAM_TYPES]
