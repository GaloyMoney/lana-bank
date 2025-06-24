with credit_facility as (
    select
        id as payment_id,
        credit_facility_id,
        amount,
        interest,
        disbursal,
        is_payment_allocated,

        * except(
            id,
            credit_facility_id,
            amount,
            interest,
            disbursal,
            is_payment_allocated,

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
    from {{ ref('stg_core_payment_events_rollup') }}
)


select * from credit_facility
