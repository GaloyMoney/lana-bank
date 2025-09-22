with latest_sequence as (
    select
        deposit_account_id,
        max(version) as version
    from {{ ref('int_core_deposit_account_events_rollup_sequence') }}
    group by deposit_account_id
),

all_event_sequence as (
    select *
    from {{ ref('int_core_deposit_account_events_rollup_sequence') }}
),

final as (
    select *
    from all_event_sequence
    inner join latest_sequence using (deposit_account_id, version)

)

select * from final
