{{ config(
    materialized = 'incremental',
    unique_key = ['id', 'sequence'],
) }}

select
    id,
    sequence,
    event_type,
    event,
    recorded_at,
    _sdc_batched_at,
    row_number()
        over (
            partition by id, sequence
            order by _sdc_received_at desc
        )
        as order_received_desc

from {{ source("lana", "credit_facility_events") }}

{% if is_incremental() %}
    where
        _sdc_batched_at >= (select coalesce(max(_sdc_batched_at), '1900-01-01') from {{ this }})
{% endif %}

qualify row_number()
    over (
        partition by id, sequence
        order by _sdc_received_at desc
    )
= 1
