with credit_facility as (
    select
        id as disbursal_id,
        credit_facility_id,
        effective,
        amount,
        approved,
        is_approval_process_concluded,
        is_settled,
        is_cancelled,
        due_date,
        overdue_date,
        liquidation_date,

        * except(
            id,
            credit_facility_id,

            effective,
            amount,
            approved,
            is_approval_process_concluded,
            is_settled,
            is_cancelled,
            due_date,
            overdue_date,
            liquidation_date,

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
    from {{ ref('stg_core_disbursal_events_rollup') }}
)


select * from credit_facility
