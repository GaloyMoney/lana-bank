with deposit_account as (
    select
        id as deposit_account_id,
        account_holder_id as customer_id,

        * except(
            id,
            account_holder_id,

            last_sequence,
            _sdc_received_at,
            _sdc_batched_at,
            _sdc_extracted_at,
            _sdc_deleted_at,
            _sdc_sequence,
            _sdc_table_version
        )
    from {{ ref('stg_core_deposit_account_events_rollup') }}
)


select * from deposit_account
