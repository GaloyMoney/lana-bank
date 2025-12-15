with latest_sequence as (
    select
        obligation_id,
        max(`version`) as `version`
    from {{ ref('int_core_obligation_events_rollup_sequence') }}
    group by obligation_id
),

all_event_sequence as (
    select *
    from {{ ref('int_core_obligation_events_rollup_sequence') }}
),

final as (
    select *
    from all_event_sequence
    inner join latest_sequence using (obligation_id, `version`)

)

select * from final
