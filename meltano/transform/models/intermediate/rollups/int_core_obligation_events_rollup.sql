with credit_facility as (
    select
        id as obligation_id,
        credit_facility_id,

        effective,
        obligation_type,
        amount,
        payment_allocation_amount,
        due_amount,
        overdue_amount,
        defaulted_amount,
        due_date,
        overdue_date,
        liquidation_date,
        defaulted_date,
        is_due_recorded,
        is_overdue_recorded,
        is_defaulted_recorded,
        is_completed,

        * except(
            id,
            credit_facility_id,

            effective,
            obligation_type,
            amount,
            payment_allocation_amount,
            due_amount,
            overdue_amount,
            defaulted_amount,
            due_date,
            overdue_date,
            liquidation_date,
            defaulted_date,
            is_due_recorded,
            is_overdue_recorded,
            is_defaulted_recorded,
            is_completed,

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
    from {{ ref('stg_core_obligation_events_rollup') }}
)


select * from credit_facility
