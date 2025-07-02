with deposit as (
    select
        id as deposit_id,
        deposit_account_id,

        cast(amount as numeric) / {{ var('cents_per_usd') }} as amount_usd,
        created_at as deposit_created_at,
        modified_at as deposit_modified_at,

        * except(
            id,
            deposit_account_id,
            amount,
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
    from {{ ref('stg_core_deposit_events_rollup') }}
)


select * from deposit
