with
    latest_sequence as (
        select deposit_account_id, max(version) as version,
        from {{ ref("int_core_deposit_account_events_rollup_sequence") }}
        group by deposit_account_id
    ),
    all_event_sequence as (
        select
            deposit_account_id,
            customer_id,
            deposit_account_created_at,
            deposit_account_modified_at,
            version,
            account_ids,
            public_id,
            status,
            loaded_to_dw_at
        from {{ ref("int_core_deposit_account_events_rollup_sequence") }}
    ),
    final as (
        select
            all_event_sequence.deposit_account_id,
            all_event_sequence.customer_id,
            all_event_sequence.deposit_account_created_at,
            all_event_sequence.deposit_account_modified_at,
            all_event_sequence.version,
            all_event_sequence.account_ids,
            all_event_sequence.public_id,
            all_event_sequence.status,
            all_event_sequence.loaded_to_dw_at
        from all_event_sequence
        inner join latest_sequence using (deposit_account_id, version)

    )

select
    deposit_account_id,
    customer_id,
    deposit_account_created_at,
    deposit_account_modified_at,
    version,
    account_ids,
    public_id,
    status,
    loaded_to_dw_at
from final
