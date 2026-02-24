{{ config(tags=["asof"]) }}

with
    latest_sequence as (
        select interest_accrual_cycle_id, max(`version`) as `version`
        from {{ ref("int_core_interest_accrual_cycle_events_rollup_sequence") }}
        where interest_accrual_cycle_modified_at <= {{ as_of_timestamp() }}
        group by interest_accrual_cycle_id
    ),

    all_event_sequence as (
        select *
        from {{ ref("int_core_interest_accrual_cycle_events_rollup_sequence") }}
    ),

    final as (
        select *
        from all_event_sequence
        inner join latest_sequence using (interest_accrual_cycle_id, `version`)

    )

select *
from final
