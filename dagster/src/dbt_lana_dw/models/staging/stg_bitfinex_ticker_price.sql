{{ config(
    unique_key ='requested_at',
) }}

with raw_bitfinex_ticker as (select * from {{ source("lana", "bitfinex_ticker_dlt")}} )
select
    * except (last_price),
    last_price as last_price_usd,
    TIMESTAMP_MICROS(CAST(CAST(_dlt_load_id AS DECIMAL) * 1e6 as INT64 )) as loaded_to_dw_at
from raw_bitfinex_ticker
