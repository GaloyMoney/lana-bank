with
    chart_initialized_at as (
        select
            coalesce(
                max(recorded_at), timestamp("1900-01-01")
            ) as initialized_recorded_at
        from {{ ref("stg_core_chart_node_events") }}
        where event_type = "initialized"
    )

select
    cast(
        json_value(
            any_value(values having max recorded_at), "$.settled.cr_balance"
        ) as numeric
    ) as settled_cr,
    cast(
        json_value(
            any_value(values having max recorded_at), "$.settled.dr_balance"
        ) as numeric
    ) as settled_dr,
    json_value(values, "$.account_id") as account_id,
    json_value(values, "$.currency") as currency

from {{ ref("stg_account_balances") }}

where recorded_at >= (select initialized_recorded_at from chart_initialized_at)

group by account_id, currency
