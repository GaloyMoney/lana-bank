select
    account_set_id,
    member_account_id as member_id,
    "Account" as member_type

from {{ ref('stg_account_set_member_accounts') }}
where _sdc_batched_at >= (
    select coalesce(max(_sdc_batched_at), '1900-01-01')
    from {{ ref('stg_core_chart_events') }}
    where event_type = 'initialized'
)
union all

select
    account_set_id,
    member_account_set_id as member_id,
    "AccountSet" as member_type

from {{ ref('stg_account_set_member_account_sets') }}
where _sdc_batched_at >= (
    select coalesce(max(_sdc_batched_at), '1900-01-01')
    from {{ ref('stg_core_chart_events') }}
    where event_type = 'initialized'
)
