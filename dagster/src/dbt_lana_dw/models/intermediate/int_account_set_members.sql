with
    chart_initialized_at as (
        select
            coalesce(
                max(recorded_at), timestamp("1900-01-01")
            ) as initialized_recorded_at
        from {{ ref("stg_core_chart_node_events") }}
        where event_type = "initialized"
    )

select account_set_id, member_account_id as member_id, "Account" as member_type

from {{ ref("stg_account_set_member_accounts") }}
where
    created_at
    >= (select initialized_recorded_at from chart_initialized_at)

union all

select account_set_id, member_account_set_id as member_id, "AccountSet" as member_type

from {{ ref("stg_account_set_member_account_sets") }}
where
    created_at
    >= (select initialized_recorded_at from chart_initialized_at)
