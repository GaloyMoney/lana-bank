{{ config(
    materialized = 'incremental',
    unique_key ='requested_at',
) }}

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
    low,
    _sdc_batched_at

from {{ source("lana", "bitfinex_ticker_view") }}

{% if is_incremental() %}
    where _sdc_batched_at >= (select coalesce(max(_sdc_batched_at), '1900-01-01') from {{ this }})
{% endif %}
