with obligation as (
    select
        id as obligation_id,
        credit_facility_id,

        cast(effective as timestamp) as effective,
        obligation_type,
        cast(amount as numeric) / {{ var('cents_per_usd') }} as amount_usd,
        cast(payment_allocation_amount as numeric) / {{ var('cents_per_usd') }} as payment_allocation_amount_usd,
        cast(due_amount as numeric) / {{ var('cents_per_usd') }} as due_amount_usd,
        cast(overdue_amount as numeric) / {{ var('cents_per_usd') }} as overdue_amount_usd,
        cast(defaulted_amount as numeric) / {{ var('cents_per_usd') }} as defaulted_amount_usd,
        cast(due_date as timestamp) as due_date,
        cast(overdue_date as timestamp) as overdue_date,
        cast(liquidation_date as timestamp) as liquidation_date,
        cast(defaulted_date as timestamp) as defaulted_date,
        is_due_recorded,
        is_overdue_recorded,
        is_defaulted_recorded,
        is_completed,
        created_at as obligation_created_at,
        modified_at as obligation_modified_at,

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
    from {{ ref('stg_core_obligation_events_rollup') }}
)


select * from obligation
