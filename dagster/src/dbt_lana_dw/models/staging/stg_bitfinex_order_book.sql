{{
    config(
        unique_key="requested_at",
    )
}}

with
    raw_bitfinex_order_book as (
        select * from {{ source("bitfinex", "bitfinex_order_book_dlt") }}
    )
select
    *,
    timestamp_micros(
        cast(cast(_dlt_load_id as decimal) * 1e6 as int64)
    ) as loaded_to_dw_at
from raw_bitfinex_order_book
