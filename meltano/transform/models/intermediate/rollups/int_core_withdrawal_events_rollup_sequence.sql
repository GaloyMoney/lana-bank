{{ config(
    materialized = 'incremental',
    unique_key = ['withdrawal_id', 'version'],
) }}


with source as (
    select s.*
    from {{ ref('stg_core_withdrawal_events_rollup') }} as s

    {% if is_incremental() %}
        left join {{ this }} as t using (withdrawal_id, version)
        where t.withdrawal_id is null
    {% endif %}
),

transformed as (
    select
        withdrawal_id,
        deposit_account_id,

        cast(amount as numeric) / {{ var('cents_per_usd') }} as amount_usd,
        approved,
        is_approval_process_concluded,
        is_confirmed,
        is_cancelled,
        created_at as withdrawal_created_at,
        modified_at as withdrawal_modified_at,

        * except (
            withdrawal_id,
            deposit_account_id,
            amount,
            approved,
            is_approval_process_concluded,
            is_confirmed,
            is_cancelled,
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
