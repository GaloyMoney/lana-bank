{{ config(
    materialized = 'incremental',
    unique_key = ['deposit_id', 'sequence'],
    full_refresh = true,
) }}


with source as (
    select
        s.*
    from {{ ref('stg_core_deposit_events_rollup') }} as s

    {% if is_incremental() %}
        left join {{ this }} as t using (deposit_id, sequence)
        where t.deposit_id is null
    {% endif %}
)


, transformed as (
    select
        deposit_id,
        deposit_account_id,

        cast(amount as numeric) / {{ var('cents_per_usd') }} as amount_usd,
        created_at as deposit_created_at,
        modified_at as deposit_modified_at,

        * except(
            deposit_id,
            deposit_account_id,
            amount,
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
