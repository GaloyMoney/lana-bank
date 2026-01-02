with
    source as (
        select
            deposit_id,
            version,
            created_at,
            modified_at,
            amount,
            deposit_account_id,
            public_id,
            reference,
            status,
            ledger_tx_ids,
            loaded_to_dw_at
        from {{ ref("stg_core_deposit_events_rollup") }} as s
    ),
    transformed as (
        select
            deposit_id,
            deposit_account_id,

            cast(amount as numeric) / 100 as amount_usd,
            created_at as deposit_created_at,
            modified_at as deposit_modified_at,

            version,
            public_id,
            reference,
            status,
            ledger_tx_ids,
            loaded_to_dw_at
        from source
    )

select
    deposit_id,
    deposit_account_id,
    amount_usd,
    deposit_created_at,
    deposit_modified_at,
    version,
    public_id,
    reference,
    status,
    ledger_tx_ids,
    loaded_to_dw_at
from transformed
