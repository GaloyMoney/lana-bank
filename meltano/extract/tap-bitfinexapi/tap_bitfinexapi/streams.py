"""Stream type classes for tap-bitfinexapi."""

from __future__ import annotations

import typing as t

from singer_sdk import typing as th  # JSON Schema typing helpers

from tap_bitfinexapi.client import BitfinexApiStream


class BitfinexTickerStream(BitfinexApiStream):
    """https://docs.bitfinex.com/reference/rest-public-ticker"""

    name = "bitfinex_ticker"
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


class BitfinexTradesStream(BitfinexApiStream):
    """https://docs.bitfinex.com/reference/rest-public-trades"""

    name = "bitfinex_trades"
    path = "/v2/trades/tBTCUSD/hist?limit=10000&sort=-1"
    primary_keys: t.ClassVar[list[str]] = ["ID"]
    schema = th.PropertiesList(
        th.Property(
            "ID",
            th.NumberType,
            description="ID of the trade",
        ),
        th.Property(
            "MTS",
            th.NumberType,
            description="Millisecond epoch timestamp",
        ),
        th.Property(
            "AMOUNT",
            th.NumberType,
            description="How much was bought (positive) or sold (negative)",
        ),
        th.Property(
            "PRICE",
            th.NumberType,
            description="Price at which the trade was executed",
        ),
    ).to_dict()


class BitfinexOrderBookStream(BitfinexApiStream):
    """https://docs.bitfinex.com/reference/rest-public-book"""

    name = "bitfinex_order_book"
    path = "/v2/book/tBTCUSD/R0?len=100"
    primary_keys: t.ClassVar[list[str]] = ["requested_at"]
    schema = th.PropertiesList(
        th.Property("requested_at", th.DateTimeType),
        th.Property(
            "orders",
            th.ArrayType(
                th.ObjectType(
                    th.Property(
                        "PRICE",
                        th.NumberType,
                        description="Price level",
                    ),
                    th.Property(
                        "COUNT",
                        th.NumberType,
                        description="Number of orders at that price level",
                    ),
                    th.Property(
                        "AMOUNT",
                        th.NumberType,
                        description="""Total amount available at that price level
(if AMOUNT > 0 then bid else ask)""",
                    ),
                ),
            ),
        ),
    ).to_dict()
