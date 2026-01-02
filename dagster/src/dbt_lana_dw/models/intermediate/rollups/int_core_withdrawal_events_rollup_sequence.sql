with
    source as (
        select
            withdrawal_id,
            version,
            created_at,
            modified_at,
            amount,
            approval_process_id,
            approved,
            deposit_account_id,
            public_id,
            reference,
            status,
            ledger_tx_ids,
            is_approval_process_concluded,
            is_cancelled,
            is_confirmed,
            loaded_to_dw_at
        from {{ ref("stg_core_withdrawal_events_rollup") }} as s
    ),
    transformed as (
        select
            withdrawal_id,
            deposit_account_id,

            cast(amount as numeric) / 100 as amount_usd,
            approved,
            is_approval_process_concluded,
            is_confirmed,
            is_cancelled,
            created_at as withdrawal_created_at,
            modified_at as withdrawal_modified_at,

            version,
            approval_process_id,
            public_id,
            reference,
            status,
            ledger_tx_ids,
            loaded_to_dw_at
        from source
    )

select
    withdrawal_id,
    deposit_account_id,
    amount_usd,
    approved,
    is_approval_process_concluded,
    is_confirmed,
    is_cancelled,
    withdrawal_created_at,
    withdrawal_modified_at,
    version,
    approval_process_id,
    public_id,
    reference,
    status,
    ledger_tx_ids,
    loaded_to_dw_at
from transformed
