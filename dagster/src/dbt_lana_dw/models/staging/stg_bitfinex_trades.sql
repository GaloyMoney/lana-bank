{{
    config(
        unique_key="ID",
    )
}}

with raw_bitfinex_trades as (select * from {{ source("lana", "bitfinex_trades_dlt") }})
select
    *,
    timestamp_micros(
        cast(cast(_dlt_load_id as decimal) * 1e6 as int64)
    ) as loaded_to_dw_at
from raw_bitfinex_trades
