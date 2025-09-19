select
    requested_at,
    bid,
    bid_size,
    ask,
    daily_change,
    daily_change_relative,
    last_price as last_price_usd,
    volume,
    high,
    low

from {{ source("lana", "bitfinex_ticker_view") }}
