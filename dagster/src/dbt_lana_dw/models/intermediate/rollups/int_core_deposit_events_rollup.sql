with
    latest_sequence as (
        select deposit_id, max(version) as version,
        from {{ ref("int_core_deposit_events_rollup_sequence") }}
        group by deposit_id
    ),
    all_event_sequence as (
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
        from {{ ref("int_core_deposit_events_rollup_sequence") }}
    ),
    final as (
        select
            all_event_sequence.deposit_id,
            all_event_sequence.deposit_account_id,
            all_event_sequence.amount_usd,
            all_event_sequence.deposit_created_at,
            all_event_sequence.deposit_modified_at,
            all_event_sequence.version,
            all_event_sequence.public_id,
            all_event_sequence.reference,
            all_event_sequence.status,
            all_event_sequence.ledger_tx_ids,
            all_event_sequence.loaded_to_dw_at
        from all_event_sequence
        inner join latest_sequence using (deposit_id, version)

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
from final
