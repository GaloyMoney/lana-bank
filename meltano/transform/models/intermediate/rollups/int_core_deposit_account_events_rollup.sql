with latest_sequence as (
    select
        deposit_account_id,
        max(sequence) as sequence,
    from {{ ref('int_core_deposit_account_events_rollup_sequence') }}
    group by deposit_account_id
)

, all_event_sequence as (
    select *
    from {{ ref('int_core_deposit_account_events_rollup_sequence') }}
)

, final as (
    select
        *
    from all_event_sequence
    inner join latest_sequence using (deposit_account_id, sequence)

)


select * from final
