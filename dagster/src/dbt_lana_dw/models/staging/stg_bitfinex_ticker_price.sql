{{
    config(
        unique_key="requested_at",
    )
}}

with
    raw_bitfinex_ticker as (
        select * from {{ source("bitfinex", "bitfinex_ticker_dlt") }}
    )
select
    * except (last_price),
    last_price as last_price_usd,
    timestamp_micros(
        cast(cast(_dlt_load_id as decimal) * 1e6 as int64)
    ) as loaded_to_dw_at
from raw_bitfinex_ticker
