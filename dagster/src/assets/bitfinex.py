from typing import Dict

import dlt

import dagster as dg
from src.core import Protoasset
from src.dlt_destinations.bigquery import create_bigquery_destination
from src.dlt_resources.bitfinex import (
    DEFAULT_ORDER_BOOK_DEPTH,
    DEFAULT_SYMBOL,
    DEFAULT_TRADES_LIMIT,
)
from src.dlt_resources.bitfinex import order_book as dlt_order_book
from src.dlt_resources.bitfinex import ticker as dlt_ticker
from src.dlt_resources.bitfinex import trades as dlt_trades
from src.resources import RESOURCE_KEY_DW_BQ, BigQueryResource


def _run_bitfinex_pipeline(
    context: dg.AssetExecutionContext,
    dw_bq: BigQueryResource,
    pipeline_name: str,
    dlt_resource,
):
    dest = create_bigquery_destination(
        credentials=dw_bq.get_credentials_dict(),
        staging_bucket=dw_bq.get_staging_bucket(),
    )

    pipe = dlt.pipeline(
        pipeline_name=pipeline_name,
        destination=dest,
        dataset_name=dw_bq.get_target_dataset(),
    )
    info = pipe.run(dlt_resource)
    context.log.info(str(info))


def bitfinex_ticker(context: dg.AssetExecutionContext, dw_bq: BigQueryResource) -> None:
    _run_bitfinex_pipeline(
        context=context,
        dw_bq=dw_bq,
        pipeline_name="bitfinex_ticker",
        dlt_resource=dlt_ticker(symbol=DEFAULT_SYMBOL),
    )


def bitfinex_trades(context: dg.AssetExecutionContext, dw_bq: BigQueryResource) -> None:
    _run_bitfinex_pipeline(
        context=context,
        dw_bq=dw_bq,
        pipeline_name="bitfinex_trades",
        dlt_resource=dlt_trades(symbol=DEFAULT_SYMBOL, limit=DEFAULT_TRADES_LIMIT),
    )


def bitfinex_order_book(
    context: dg.AssetExecutionContext, dw_bq: BigQueryResource
) -> None:
    _run_bitfinex_pipeline(
        context=context,
        dw_bq=dw_bq,
        pipeline_name="bitfinex_order_book",
        dlt_resource=dlt_order_book(
            symbol=DEFAULT_SYMBOL, depth=DEFAULT_ORDER_BOOK_DEPTH
        ),
    )


def bitfinex_protoassets() -> Dict[str, Protoasset]:
    """Return all Bitfinex protoassets keyed by asset key."""
    return {
        "bitfinex_ticker": Protoasset(
            key=dg.AssetKey("bitfinex_ticker"),
            callable=bitfinex_ticker,
            required_resource_keys={RESOURCE_KEY_DW_BQ},
        ),
        "bitfinex_trades": Protoasset(
            key=dg.AssetKey("bitfinex_trades"),
            callable=bitfinex_trades,
            required_resource_keys={RESOURCE_KEY_DW_BQ},
        ),
        "bitfinex_order_book": Protoasset(
            key=dg.AssetKey("bitfinex_order_book"),
            callable=bitfinex_order_book,
            required_resource_keys={RESOURCE_KEY_DW_BQ},
        ),
    }
