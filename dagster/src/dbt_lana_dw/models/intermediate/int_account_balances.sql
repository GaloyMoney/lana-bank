with
    chart_initialized_at as (
        select
            coalesce(
                max(recorded_at), timestamp("1900-01-01")
            ) as initialized_recorded_at
        from {{ ref("stg_core_chart_node_events") }}
        where event_type = "initialized"
    ),

    ranked as (
        select
            json_value(values, "$.account_id") as account_id,
            json_value(values, "$.currency") as currency,
            cast(json_value(values, "$.settled.cr_balance") as numeric) as settled_cr,
            cast(json_value(values, "$.settled.dr_balance") as numeric) as settled_dr,
            row_number() over (
                partition by
                    json_value(values, "$.account_id"), json_value(values, "$.currency")
                order by version desc
            ) as rn
        from {{ ref("stg_account_balances") }}
        where recorded_at >= (select initialized_recorded_at from chart_initialized_at)
    )

select account_id, currency, settled_cr, settled_dr
from ranked
where rn = 1
