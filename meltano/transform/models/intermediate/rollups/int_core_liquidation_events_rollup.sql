with latest_sequence as (
    select
        liquidation_id,
        max(`version`) as `version`
    from {{ ref('int_core_liquidation_events_rollup_sequence') }}
    group by liquidation_id
),

all_event_sequence as (
    select *
    from {{ ref('int_core_liquidation_events_rollup_sequence') }}
),

final as (
    select *
    from all_event_sequence
    inner join latest_sequence using (liquidation_id, `version`)

)

select * from final
