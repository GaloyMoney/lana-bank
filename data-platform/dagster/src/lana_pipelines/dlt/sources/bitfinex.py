from __future__ import annotations

from datetime import datetime, timezone
from decimal import Decimal
from typing import Iterable, Iterator, Dict, Any, List

import dlt
from dlt.sources.helpers import requests

BITFINEX_API_BASE = "https://api-pub.bitfinex.com"
DEFAULT_SYMBOL = "tBTCUSD"
DEFAULT_ORDER_BOOK_DEPTH = 100
DEFAULT_TRADES_LIMIT = 10000

# https://docs.bitfinex.com/reference/rest-public-ticker
FIELDS_TICKER = [
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
]

def _utc_now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()

@dlt.resource(
    name="bitfinex_ticker_dlt",
    write_disposition="append",
    primary_key="requested_at",
)
def ticker(symbol: str = DEFAULT_SYMBOL) -> Iterator[Dict[str, Any]]:
    """
    Snapshot ticker for a given symbol.

    - Appends a new row on each run.
    - Primary key is requested_at (snapshot timestamps).
    """
    url = f"{BITFINEX_API_BASE}/v2/ticker/{symbol}"
    resp = requests.get(url)
    resp.raise_for_status()

    data = resp.json(parse_float=Decimal)
    yield dict(zip(FIELDS_TICKER, data)) | {
        "symbol": symbol,
        "requested_at": _utc_now_iso(),
    }

def today_midnight_utc_ms():
    now = datetime.now(timezone.utc)
    midnight = now.replace(hour=0, minute=0, second=0, microsecond=0)
    return int(midnight.timestamp() * 1000)

@dlt.resource(
    name="bitfinex_trades_dlt",
    write_disposition="append",
    primary_key="ID",
)
def trades(
    symbol: str = DEFAULT_SYMBOL,
    limit: int = DEFAULT_TRADES_LIMIT,
    mts=dlt.sources.incremental("MTS", initial_value=today_midnight_utc_ms()),
) -> Iterator[Dict[str, Any]]:
    """
    Incremental trade history for a given symbol.

    - Uses MTS (ms epoch) as incremental cursor to fetch forward in time.
    - Merges on ID for idempotency when the API returns overlapping windows.
    """
    start = int(mts.last_value or 0)
    url = f"{BITFINEX_API_BASE}/v2/trades/{symbol}/hist"

    while True:
        params = {"limit": limit, "sort": 1, "start": start}  # asc order
        resp = requests.get(url, params=params)
        resp.raise_for_status()

        rows: List[List[Any]] = resp.json(parse_float=Decimal)
        if not rows:
            break

        for row in rows:
            record = dict(zip(["ID", "MTS", "AMOUNT", "PRICE"], row))
            record["symbol"] = symbol
            yield record

        # advance window strictly beyond the last seen MTS to avoid duplicates
        start = int(rows[-1][1]) + 1
        # If fewer than requested, we've reached the end
        if len(rows) < limit:
            break

@dlt.resource(
    name="bitfinex_order_book_dlt",
    write_disposition="append",
    primary_key="requested_at",
)
def order_book(symbol: str = DEFAULT_SYMBOL, depth: int = DEFAULT_ORDER_BOOK_DEPTH) -> Iterator[Dict[str, Any]]:
    """
    Snapshot full order book for a given symbol.

    - Appends one parent record with nested 'orders' array per run.
    - dlt will create a child table bitfinex_order_book__orders automatically.
    """
    url = f"{BITFINEX_API_BASE}/v2/book/{symbol}/R0"
    resp = requests.get(url, params={"len": depth})
    resp.raise_for_status()

    orders = [
        dict(zip(["PRICE", "COUNT", "AMOUNT"], o)) for o in resp.json(parse_float=Decimal)
    ]
    yield {
        "symbol": symbol,
        "requested_at": _utc_now_iso(),
        "orders": orders,
    }

@dlt.source
def bitfinex_source(
    symbol: str = DEFAULT_SYMBOL,
    depth: int = DEFAULT_ORDER_BOOK_DEPTH,
    trades_limit: int = DEFAULT_TRADES_LIMIT,
):
    """
    Grouped Bitfinex source returning ticker, trades, and order_book resources.
    """
    return [
        ticker(symbol),
        trades(symbol, limit=trades_limit),
        order_book(symbol, depth),
    ]
