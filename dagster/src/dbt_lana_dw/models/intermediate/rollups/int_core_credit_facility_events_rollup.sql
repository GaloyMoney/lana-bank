with
    latest_sequence as (
        select
            credit_facility_id,
            max(`version`) as `version`,
            max(`proposal_version`) as `proposal_version`
        from {{ ref("int_core_credit_facility_events_rollup_sequence") }}
        group by credit_facility_id
    ),

    all_event_sequence as (
        select * from {{ ref("int_core_credit_facility_events_rollup_sequence") }}
    ),

    final as (
        select *
        from all_event_sequence
        inner join
            latest_sequence using (credit_facility_id, `version`, `proposal_version`)

    )

select *
from final
