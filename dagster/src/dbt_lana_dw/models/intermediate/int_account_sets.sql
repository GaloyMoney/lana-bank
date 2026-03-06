with
    chart_initialized_at as (
        select
            coalesce(
                max(recorded_at), timestamp("1900-01-01")
            ) as initialized_recorded_at
        from {{ ref("stg_core_chart_node_events") }}
        where event_type = "initialized"
    )

select id as account_set_id, set_name, row_number() over () as set_key

from {{ ref("stg_account_sets") }}
where created_at >= (select initialized_recorded_at from chart_initialized_at)
