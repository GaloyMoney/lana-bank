{{ config(
    materialized = 'incremental',
    unique_key = ['id', 'version'],
) }}

select
    s.id as payment_id,
    s.*
from {{ ref('public_core_payment_events_rollup_view') }} as s

{% if is_incremental() %}
    left join {{ this }} as t using (id, version)
    where s._sdc_batched_at = (select max(_sdc_batched_at) from {{ ref('public_core_payment_events_rollup_view') }})
    and t.id is null
{% else %}
    where s._sdc_batched_at = (select max(_sdc_batched_at) from {{ ref('public_core_payment_events_rollup_view') }})
{% endif %}
