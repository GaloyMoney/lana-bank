{{ config(
    unique_key ='ID',
    full_refresh = true,
) }}

select
    id,
    mts,
    amount,
    price,
    _sdc_batched_at

from {{ source("lana", "bitfinex_trades_view") }}
