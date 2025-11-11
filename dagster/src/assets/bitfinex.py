import dlt
import dagster as dg

from src.dlt_resources.bitfinex import (
    ticker as dlt_ticker,
    trades as dlt_trades,
    order_book as dlt_order_book,
    DEFAULT_SYMBOL,
    DEFAULT_ORDER_BOOK_DEPTH,
    DEFAULT_TRADES_LIMIT,
)
from src.dlt_destinations.bigquery import create_bigquery_destination

def _bq_config():
    base64_credentials = dg.EnvVar("TF_VAR_sa_creds").get_value()
    target_dataset = dg.EnvVar("TARGET_BIGQUERY_DATASET").get_value()
    if not base64_credentials:
        raise RuntimeError("TF_VAR_sa_creds is not set")
    if not target_dataset:
        raise RuntimeError("TARGET_BIGQUERY_DATASET is not set")
    return base64_credentials, target_dataset

def bitfinex_ticker(context: dg.AssetExecutionContext) -> None:
    symbol = DEFAULT_SYMBOL
    base64_credentials, dataset = _bq_config()
    dest = create_bigquery_destination(base64_credentials)

    pipe = dlt.pipeline(pipeline_name="bitfinex_ticker", destination=dest, dataset_name=dataset)
    info = pipe.run(dlt_ticker(symbol=symbol))
    context.log.info(str(info))

def bitfinex_trades(context: dg.AssetExecutionContext) -> None:
    symbol = DEFAULT_SYMBOL
    trades_limit = DEFAULT_TRADES_LIMIT
    base64_credentials, dataset = _bq_config()
    dest = create_bigquery_destination(base64_credentials)

    pipe = dlt.pipeline(pipeline_name="bitfinex_trades", destination=dest, dataset_name=dataset)
    info = pipe.run(dlt_trades(symbol=symbol, limit=trades_limit))
    context.log.info(str(info))

def bitfinex_order_book(context: dg.AssetExecutionContext) -> None:
    symbol = DEFAULT_SYMBOL
    depth = DEFAULT_ORDER_BOOK_DEPTH
    base64_credentials, dataset = _bq_config()
    dest = create_bigquery_destination(base64_credentials)

    pipe = dlt.pipeline(pipeline_name="bitfinex_order_book", destination=dest, dataset_name=dataset)
    info = pipe.run(dlt_order_book(symbol=symbol, depth=depth))
    context.log.info(str(info))
