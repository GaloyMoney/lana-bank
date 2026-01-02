select account_set_id, member_account_id as member_id, "Account" as member_type

from {{ ref("stg_account_set_member_accounts") }}
where
    loaded_to_dw_at >= (
        select coalesce(max(loaded_to_dw_at), "1900-01-01")
        from {{ ref("stg_core_chart_node_events") }}
        where event_type = "initialized"
    )

union all

select account_set_id, member_account_set_id as member_id, "AccountSet" as member_type

from {{ ref("stg_account_set_member_account_sets") }}
where
    loaded_to_dw_at >= (
        select coalesce(max(loaded_to_dw_at), "1900-01-01")
        from {{ ref("stg_core_chart_node_events") }}
        where event_type = "initialized"
    )
