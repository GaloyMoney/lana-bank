with latest_sequence as (
    select
        credit_facility_proposal_id,
        max(version) as max_version
    from {{ ref('int_core_credit_facility_proposal_events_rollup_sequence') }}
    group by credit_facility_proposal_id
),

all_event_sequence as (
    select *
    from {{ ref('int_core_credit_facility_proposal_events_rollup_sequence') }}
),

final as (
    select aes.*
    from all_event_sequence as aes
    inner join latest_sequence as ls
        on
            aes.credit_facility_proposal_id = ls.credit_facility_proposal_id
            and aes.version = ls.max_version
)

select * from final
