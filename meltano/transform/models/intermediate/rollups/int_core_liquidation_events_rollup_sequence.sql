{{ config(
    materialized = 'incremental',
    unique_key = ['liquidation_id', 'version'],
) }}


with source as (
    select s.*
    from {{ ref('stg_core_liquidation_events_rollup') }} as s

    {% if is_incremental() %}
        left join {{ this }} as t using (liquidation_id, version)
        where t.liquidation_id is null
    {% endif %}
),

transformed as (
    select
        * except (
            liquidation_id,
            credit_facility_id,

            is_completed,
            created_at,
            modified_at,

            _sdc_received_at,
            _sdc_batched_at,
            _sdc_extracted_at,
            _sdc_deleted_at,
            _sdc_sequence,
            _sdc_table_version
        ),
        liquidation_id,

        credit_facility_id,
        is_completed,
        created_at as liquidation_created_at,

        modified_at as liquidation_modified_at
    from source
)

select * from transformed
