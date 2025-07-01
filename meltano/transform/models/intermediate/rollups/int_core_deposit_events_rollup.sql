with deposit as (
    select
        id as deposit_id,
        deposit_account_id,

        cast(amount as numeric) / {{ var('cents_per_usd') }} as amount_usd,

        * except(
            id,
            deposit_account_id,
            amount,

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
