{{ config(tags=["asof"]) }}

with
    latest_effective as (
        select account_id, currency, max(effective) as effective
        from {{ ref("stg_cumulative_effective_balances") }}
        where effective <= {{ as_of_date() }}
        group by account_id, currency
    )

select
    cast(json_value(bal.values, "$.settled.cr_balance") as numeric) as settled_cr,
    cast(json_value(bal.values, "$.settled.dr_balance") as numeric) as settled_dr,
    bal.account_id,
    bal.currency

from {{ ref("stg_cumulative_effective_balances") }} as bal
inner join latest_effective as le using (account_id, currency, effective)

where
    bal.loaded_to_dw_at >= (
        select coalesce(max(loaded_to_dw_at), "1900-01-01")
        from {{ ref("stg_core_chart_node_events") }}
        where event_type = "initialized"
    )
