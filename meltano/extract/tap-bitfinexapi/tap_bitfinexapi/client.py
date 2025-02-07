"""REST client handling, including BitfinexApiStream base class."""

from __future__ import annotations

import decimal
import typing as t
from datetime import datetime

from singer_sdk.streams import RESTStream

if t.TYPE_CHECKING:
    import requests
    from singer_sdk.helpers.types import Context


class BitfinexApiStream(RESTStream):
    """BitfinexApi stream class."""

    url_base = "https://api-pub.bitfinex.com"

    def parse_response(self, response: requests.Response) -> t.Iterable[dict]:
        """Parse the response and return an iterator of result records.

        Args:
            response: The HTTP ``requests.Response`` object.

        Yields:
            Each record from the source.
        """
        if "ticker" in response.url:
            yield dict(
                zip(
                    [
                        "BID",
                        "BID_SIZE",
                        "ASK",
                        "ASK_SIZE",
                        "DAILY_CHANGE",
                        "DAILY_CHANGE_RELATIVE",
                        "LAST_PRICE",
                        "VOLUME",
                        "HIGH",
                        "LOW",
                    ],
                    response.json(parse_float=decimal.Decimal),
                )
            ) | {"requested_at": datetime.now().isoformat()}
        elif "trades" in response.url:
            for trade in response.json(parse_float=decimal.Decimal):
                yield dict(
                    zip(
                        [
                            "ID",
                            "MTS",
                            "AMOUNT",
                            "PRICE",
                        ],
                        trade,
                    )
                )
        elif "book" in response.url:
            book = {"requested_at": datetime.now().isoformat(), "orders": []}
            for order in response.json(parse_float=decimal.Decimal):
                book["orders"].append(
                    dict(
                        zip(
                            [
                                "PRICE",
                                "COUNT",
                                "AMOUNT",
                            ],
                            order,
                        )
                    )
                )
            yield book
