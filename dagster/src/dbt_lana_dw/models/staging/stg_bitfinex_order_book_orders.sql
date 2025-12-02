with raw_bitfinex_order_book_dlt__orders as (select * from {{ source("lana", "bitfinex_order_book_dlt__orders")}} )
select
    *
from raw_bitfinex_order_book_dlt__orders
