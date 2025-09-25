{{ config(
    materialized = 'incremental',
    unique_key = ['id', 'version'],
) }}

select
    s.*,
    s.id as disbursal_id,
    s.facility_id as credit_facility_id
from {{ source("lana", "public_core_disbursal_events_rollup_view") }} as s

{% if is_incremental() %}
    left join {{ this }} as t using (id, version)
    where
        s._sdc_batched_at
        = (
            select max(_sdc_batched_at)
            from {{ source("lana", "public_core_disbursal_events_rollup_view") }}
        )
        and t.id is null
{% else %}
    where
        s._sdc_batched_at
        = (
            select max(_sdc_batched_at)
            from {{ source("lana", "public_core_disbursal_events_rollup_view") }}
        )
{% endif %}
