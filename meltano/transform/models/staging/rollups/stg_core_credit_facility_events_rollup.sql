{{ config(
    materialized = 'incremental',
    unique_key = ['id', 'sequence'],
    full_refresh = true,
) }}

select
    s.id as credit_facility_id,
    s.*
from {{ source("lana", "public_core_credit_facility_events_rollup_view") }} as s

{% if is_incremental() %}
    left join {{ this }} as t using (id, sequence)
    where s._sdc_batched_at = (select max(_sdc_batched_at) from {{ source("lana", "public_core_credit_facility_events_rollup_view") }})
    and t.id is null
{% else %}
    where s._sdc_batched_at = (select max(_sdc_batched_at) from {{ source("lana", "public_core_credit_facility_events_rollup_view") }})
{% endif %}
