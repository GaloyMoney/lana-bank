{{ config(
    materialized = 'incremental',
    unique_key = ['credit_facility_proposal_id', 'version'],
) }}

with source as (
    select s.*
    from {{ ref('stg_core_credit_facility_proposal_events_rollup') }} as s
    {% if is_incremental() %}
        left join {{ this }} as t using (credit_facility_proposal_id, version)
        where t.credit_facility_proposal_id is null
    {% endif %}
),

transformed as (
    select
        * except (
            credit_facility_proposal_id,
            version,
            _sdc_received_at,
            _sdc_batched_at,
            _sdc_extracted_at,
            _sdc_deleted_at,
            _sdc_sequence,
            _sdc_table_version
        ),
        credit_facility_proposal_id,
        version
    from source
),

final as (
    select *
    from transformed
)

select * from final
