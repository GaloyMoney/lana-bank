{{ config(materialized="view") }}

with
    chart as (select * from {{ ref("int_core_chart_of_accounts") }}),

    chart_dates as (
        select date_spine.as_of_date, chart.*
        from chart
        cross join {{ ref("int_date_spine") }} as date_spine
    ),

    balances as (
        select as_of_date, account_set_id, coalesce(sum(balance), 0) as balance
        from {{ ref("int_account_sets_expanded_with_balances_daily") }}
        group by as_of_date, account_set_id
    ),

    final as (
        select
            chart_dates.as_of_date,
            chart_dates.code,
            chart_dates.dotted_code,
            chart_dates.spaced_code,
            chart_dates.node_name,
            chart_dates.account_set_id,
            coalesce(balances.balance, 0) as balance
        from chart_dates
        left join
            balances
            on chart_dates.account_set_id = balances.account_set_id
            and chart_dates.as_of_date = balances.as_of_date
    )

select *
from final
