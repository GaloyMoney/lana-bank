with latest_sequence as (
    select
        pending_credit_facility_id,
        max(version) as max_version
    from {{ ref('int_core_pending_credit_facility_events_rollup_sequence') }}
    group by pending_credit_facility_id
),

all_event_sequence as (
    select *
    from {{ ref('int_core_pending_credit_facility_events_rollup_sequence') }}
),

final as (
    select aes.*
    from all_event_sequence as aes
    inner join latest_sequence as ls
        on
            aes.pending_credit_facility_id = ls.pending_credit_facility_id
            and aes.version = ls.max_version
)

select * from final
