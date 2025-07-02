with payment as (
    select
        id as payment_id,
        credit_facility_id,
        cast(amount as numeric) / {{ var('cents_per_usd') }} as amount_usd,
        cast(interest as numeric) / {{ var('cents_per_usd') }} as interest_usd,
        cast(disbursal as numeric) / {{ var('cents_per_usd') }} as disbursal_usd,
        is_payment_allocated,
        created_at as payment_created_at,
        modified_at as payment_modified_at,

        * except(
            id,
            credit_facility_id,
            amount,
            interest,
            disbursal,
            is_payment_allocated,
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
    from {{ ref('stg_core_payment_events_rollup') }}
)


select * from payment
