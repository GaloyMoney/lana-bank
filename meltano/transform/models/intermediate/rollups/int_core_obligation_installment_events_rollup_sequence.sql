{{ config(
    materialized = 'incremental',
    unique_key = ['obligation_installment_id', 'version'],
) }}


with source as (
    select
        s.*
    from {{ ref('stg_core_obligation_installment_events_rollup') }} as s

    {% if is_incremental() %}
        left join {{ this }} as t using (obligation_installment_id, version)
        where t.obligation_installment_id is null
    {% endif %}
)


, transformed as (
    select
        obligation_installment_id,
        payment_id,
        credit_facility_id,
        cast(amount as numeric) / {{ var('cents_per_usd') }} as amount_usd,
        cast(effective as timestamp) as effective,
        obligation_type,
        obligation_installment_idx,
        account_to_be_debited_id,
        receivable_account_id,
        obligation_id,
        created_at as obligation_installment_created_at,
        modified_at as obligation_installment_modified_at,

        * except(
            obligation_installment_id,
            payment_id,
            credit_facility_id,
            amount,
            effective,
            obligation_type,
            obligation_installment_idx,
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
