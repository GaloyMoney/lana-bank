{{ config(
    materialized = 'incremental',
    unique_key ='requested_at',
) }}

select
    requested_at,
    orders,
    _sdc_batched_at

from {{ source("lana", "bitfinex_order_book_view") }}

{% if is_incremental() %}
    where _sdc_batched_at >= (select coalesce(max(_sdc_batched_at), '1900-01-01') from {{ this }})
{% endif %}
