{{ config(
    materialized = 'incremental',
    unique_key = ['pending_credit_facility_id', 'version'],
) }}

with source as (
    select s.*
    from {{ ref('stg_core_pending_credit_facility_events_rollup') }} as s
    {% if is_incremental() %}
        left join {{ this }} as t using (pending_credit_facility_id, version)
        where t.pending_credit_facility_id is null
    {% endif %}
),

transformed as (
    select
        * except (
            pending_credit_facility_id,
            version,
            _sdc_received_at,
            _sdc_batched_at,
            _sdc_extracted_at,
            _sdc_deleted_at,
            _sdc_sequence,
            _sdc_table_version
        ),
        pending_credit_facility_id,
        version
    from source
),

final as (
    select *
    from transformed
)

select * from final
