with raw_bitfinex_trades as (select * from {{ source("lana", "bitfinex_trades_dlt")}} )
select
    *,
    TIMESTAMP_MICROS(CAST(CAST(_dlt_load_id AS DECIMAL) * 1e6 as INT64 )) as loaded_to_dw_at
from raw_bitfinex_trades
