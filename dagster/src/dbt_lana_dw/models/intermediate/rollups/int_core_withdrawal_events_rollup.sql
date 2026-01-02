with
    latest_sequence as (
        select withdrawal_id, max(version) as version,
        from {{ ref("int_core_withdrawal_events_rollup_sequence") }}
        group by withdrawal_id
    ),
    all_event_sequence as (
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
        from {{ ref("int_core_withdrawal_events_rollup_sequence") }}
    ),
    final as (
        select
            all_event_sequence.withdrawal_id,
            all_event_sequence.deposit_account_id,
            all_event_sequence.amount_usd,
            all_event_sequence.approved,
            all_event_sequence.is_approval_process_concluded,
            all_event_sequence.is_confirmed,
            all_event_sequence.is_cancelled,
            all_event_sequence.withdrawal_created_at,
            all_event_sequence.withdrawal_modified_at,
            all_event_sequence.version,
            all_event_sequence.approval_process_id,
            all_event_sequence.public_id,
            all_event_sequence.reference,
            all_event_sequence.status,
            all_event_sequence.ledger_tx_ids,
            all_event_sequence.loaded_to_dw_at
        from all_event_sequence
        inner join latest_sequence using (withdrawal_id, version)

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
from final
