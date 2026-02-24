{{ config(tags=["asof"]) }}

with
    latest_sequence as (
        select payment_allocation_id, max(`version`) as `version`
        from {{ ref("int_core_payment_allocation_events_rollup_sequence") }}
        where payment_allocation_modified_at <= {{ as_of_timestamp() }}
        group by payment_allocation_id
    ),

    all_event_sequence as (
        select * from {{ ref("int_core_payment_allocation_events_rollup_sequence") }}
    ),

    final as (
        select *
        from all_event_sequence
        inner join latest_sequence using (payment_allocation_id, `version`)

    )

select *
from final
