select
    id,
    mts,
    amount,
    price

from {{ source("lana", "bitfinex_trades_view") }}
