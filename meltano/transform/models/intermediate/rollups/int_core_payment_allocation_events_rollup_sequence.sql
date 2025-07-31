{{ config(
    materialized = 'incremental',
    unique_key = ['payment_allocation_id', 'version'],
    full_refresh = true,
) }}


with source as (
    select
        s.*
    from {{ ref('stg_core_payment_allocation_events_rollup') }} as s

    {% if is_incremental() %}
        left join {{ this }} as t using (payment_allocation_id, version)
        where t.payment_allocation_id is null
    {% endif %}
)


, transformed as (
    select
        payment_allocation_id,
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
            payment_allocation_id,
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
