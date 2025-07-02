with payment_allocation as (
    select
        id as payment_allocation_id,
        payment_id,
        credit_facility_id,
        cast(amount as numeric) / {{ var('cents_per_usd') }} as amount_usd,
        cast(effective as timestamp) as effective,
        obligation_type,
        obligation_allocation_idx,
        account_to_be_debited_id,
        receivable_account_id,
        obligation_id,
        created_at as payment_allocation_created_at,
        modified_at as payment_allocation_modified_at,

        * except(
            id,
            payment_id,
            credit_facility_id,
            amount,
            effective,
            obligation_type,
            obligation_allocation_idx,
            account_to_be_debited_id,
            receivable_account_id,
            obligation_id,
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
    from {{ ref('stg_core_payment_allocation_events_rollup') }}
)


select * from payment_allocation
