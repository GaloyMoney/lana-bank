with latest_sequence as (
    select
        deposit_id,
        max(version) as version
    from {{ ref('int_core_deposit_events_rollup_sequence') }}
    group by deposit_id
),

all_event_sequence as (
    select *
    from {{ ref('int_core_deposit_events_rollup_sequence') }}
),

final as (
    select *
    from all_event_sequence
    inner join latest_sequence using (deposit_id, version)

)

select * from final
