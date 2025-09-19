with latest_sequence as (
    select
        obligation_installment_id,
        max(version) as version
    from {{ ref('int_core_obligation_installment_events_rollup_sequence') }}
    group by obligation_installment_id
),

all_event_sequence as (
    select *
    from {{ ref('int_core_obligation_installment_events_rollup_sequence') }}
),

final as (
    select *
    from all_event_sequence
    inner join latest_sequence using (obligation_installment_id, version)

)

select * from final
