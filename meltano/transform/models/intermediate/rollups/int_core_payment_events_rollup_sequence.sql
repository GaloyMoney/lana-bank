{{ config(
    materialized = 'incremental',
    unique_key = ['payment_id', 'sequence'],
    full_refresh = true,
) }}


with source as (
    select
        s.*
    from {{ ref('stg_core_payment_events_rollup') }} as s

    {% if is_incremental() %}
        left join {{ this }} as t using (payment_id, sequence)
        where t.payment_id is null
    {% endif %}
)


, transformed as (
    select
        payment_id,
        credit_facility_id,
        cast(amount as numeric) / {{ var('cents_per_usd') }} as amount_usd,
        cast(interest as numeric) / {{ var('cents_per_usd') }} as interest_usd,
        cast(disbursal as numeric) / {{ var('cents_per_usd') }} as disbursal_usd,
        is_payment_allocated,
        created_at as payment_created_at,
        modified_at as payment_modified_at,

        * except(
            payment_id,
            credit_facility_id,
            amount,
            interest,
            disbursal,
            is_payment_allocated,
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
