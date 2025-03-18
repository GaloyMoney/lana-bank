select
    date(requested_at) as day,
{% if target.type == 'bigquery' %}
    any_value(last_price_usd having max requested_at) as close_price_usd_per_btc
{% elif target.type == 'snowflake' %}
    GET(MAX_BY(last_price_usd, requested_at, 1), 0) as close_price_usd_per_btc
{% endif %}

from {{ ref('stg_bitfinex_ticker_price') }}

group by day
