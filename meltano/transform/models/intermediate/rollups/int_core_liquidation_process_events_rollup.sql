with liquidation_process as (
    select
        id as liquidation_process_id,
        credit_facility_id,

        cast(effective as timestamp) as effective,
        is_completed,
        cast(initial_amount as numeric) / {{ var('cents_per_usd') }} as initial_amount_usd,
        created_at as liquidation_process_created_at,
        modified_at as liquidation_process_modified_at,

        * except(
            id,
            credit_facility_id,

            effective,
            is_completed,
            initial_amount,
            created_at,
            modified_at,

            last_sequence,
            _sdc_received_at,
            _sdc_batched_at,
            _sdc_extracted_at,
            _sdc_deleted_at,
            _sdc_sequence,
            _sdc_table_version
        )
    from {{ ref('stg_core_liquidation_process_events_rollup') }}
)


select * from liquidation_process
