{{ config(
    materialized = 'incremental',
    unique_key ='ID',
    full_refresh = true,
) }}
-- TODO: turn off full_refresh after rollout

select
    id,
    mts,
    amount,
    price,
    _sdc_batched_at

from {{ source("lana", "bitfinex_trades_view") }}

{% if is_incremental() %}
where _sdc_batched_at >= (select coalesce(max(_sdc_batched_at),'1900-01-01') from {{ this }} )
{% endif %}
