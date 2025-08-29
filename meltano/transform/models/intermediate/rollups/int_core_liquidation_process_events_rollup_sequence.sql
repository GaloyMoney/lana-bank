{{ config(
    materialized = 'incremental',
    unique_key = ['liquidation_process_id', 'version'],
) }}


with source as (
    select
        s.*
    from {{ ref('stg_core_liquidation_process_events_rollup') }} as s

    {% if is_incremental() %}
        left join {{ this }} as t using (liquidation_process_id, version)
        where t.liquidation_process_id is null
    {% endif %}
)


, transformed as (
    select
        liquidation_process_id,
        credit_facility_id,

        cast(effective as timestamp) as effective,
        is_completed,
        cast(initial_amount as numeric) / {{ var('cents_per_usd') }} as initial_amount_usd,
        created_at as liquidation_process_created_at,
        modified_at as liquidation_process_modified_at,

        * except(
            liquidation_process_id,
            credit_facility_id,

            effective,
            is_completed,
            initial_amount,
            created_at,
            modified_at,

            _sdc_received_at,
            _sdc_batched_at,
            _sdc_extracted_at,
            _sdc_deleted_at,
            _sdc_sequence,
            _sdc_table_version
        )
    from source
)


select * from transformed
