with latest_sequence as (
    select
        withdrawal_id,
        max(sequence) as sequence,
    from {{ ref('int_core_withdrawal_events_rollup_sequence') }}
    group by withdrawal_id
)

, all_event_sequence as (
    select *
    from {{ ref('int_core_withdrawal_events_rollup_sequence') }}
)

, final as (
    select
        *
    from all_event_sequence
    inner join latest_sequence using (withdrawal_id, sequence)

)


select * from final
