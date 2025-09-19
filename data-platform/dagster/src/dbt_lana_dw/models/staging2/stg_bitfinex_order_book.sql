select
    requested_at,
    orders

from {{ source("lana", "bitfinex_order_book_view") }}
