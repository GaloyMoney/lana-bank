with latest_sequence as (
    select
        collateral_id,
        max(sequence) as sequence,
    from {{ ref('int_core_collateral_events_rollup_sequence') }}
    group by collateral_id
)

, all_event_sequence as (
    select *
    from {{ ref('int_core_collateral_events_rollup_sequence') }}
)

, final as (
    select
        *
    from all_event_sequence
    inner join latest_sequence using (collateral_id, sequence)

)


select * from final
