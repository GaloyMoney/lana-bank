with credit_facility as (
    select
        id as liquidation_process_id,
        credit_facility_id,

        effective,
        is_completed,
        initial_amount,

        * except(
            id,
            credit_facility_id,

            effective,
            is_completed,
            initial_amount,

            last_sequence,
            created_at,
            modified_at,
            _sdc_received_at,
            _sdc_batched_at,
            _sdc_extracted_at,
            _sdc_deleted_at,
            _sdc_sequence,
            _sdc_table_version
        )
    from {{ ref('stg_core_liquidation_process_events_rollup') }}
)


select * from credit_facility
