"""Stream type classes for tap-bitfinexapi."""

from __future__ import annotations

import typing as t
from importlib import resources

from singer_sdk import typing as th  # JSON Schema typing helpers

from tap_bitfinexapi.client import BitfinexApiStream

class TickerStream(BitfinexApiStream):
    """Ticker
    https://docs.bitfinex.com/reference/rest-public-ticker
    """
    name = "ticker"
    path = "/v2/ticker/tBTCUSD"
    primary_keys: t.ClassVar[list[str]] = ["requested_at"]
    schema = th.PropertiesList(
        th.Property("requested_at", th.DateTimeType),
        th.Property(
            "BID",
            th.NumberType,
            description="Price of last highest bid",
        ),
        th.Property(
            "BID_SIZE",
            th.NumberType,
            description="Sum of the 25 highest bid sizes",
        ),
        th.Property(
            "ASK",
            th.NumberType,
            description="Price of last lowest ask",
        ),
        th.Property(
            "ASK_SIZE",
            th.NumberType,
            description="Sum of the 25 lowest ask sizes",
        ),
        th.Property(
            "DAILY_CHANGE",
            th.NumberType,
            description="Amount that the last price has changed since yesterday",
        ),
        th.Property(
            "DAILY_CHANGE_RELATIVE",
            th.NumberType,
            description="Relative price change since yesterday (*100 for percentage change)",
        ),
        th.Property(
            "LAST_PRICE",
            th.NumberType,
            description="Price of the last trade",
        ),
        th.Property(
            "VOLUME",
            th.NumberType,
            description="Daily volume",
        ),
        th.Property(
            "HIGH",
            th.NumberType,
            description="Daily high",
        ),
        th.Property(
            "LOW",
            th.NumberType,
            description="Daily low",
        ),
    ).to_dict()
