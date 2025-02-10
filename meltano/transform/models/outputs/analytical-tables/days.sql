select
    date(requested_at) as day,
    any_value(last_price_usd having max requested_at) as close_price_usd

from {{ ref('stg_bitfinex_ticker_price') }}

group by day
