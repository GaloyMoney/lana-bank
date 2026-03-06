with
    latest_sequence as (
        select party_id, max(`version`) as `version`
        from {{ ref("int_core_party_events_rollup_sequence") }}
        group by party_id
    ),

    all_event_sequence as (
        select * from {{ ref("int_core_party_events_rollup_sequence") }}
    ),

    final as (
        select *
        from all_event_sequence
        inner join latest_sequence using (party_id, `version`)

    )

select *
from final
