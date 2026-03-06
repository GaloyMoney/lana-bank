{{
    config(
        materialized="table",
        partition_by={"field": "as_of_date", "data_type": "date"},
        cluster_by=["account_id", "currency"],
    )
}}

with
    chart_initialized_at as (
        select
            coalesce(
                max(recorded_at), timestamp("1900-01-01")
            ) as initialized_recorded_at
        from {{ ref("stg_core_chart_node_events") }}
        where event_type = "initialized"
    ),

    cumulative_effective_balances as (
        select account_id, currency, effective, version, all_time_version, `values`
        from {{ ref("stg_cumulative_effective_balances") }}
        where
            effective
            >= date((select initialized_recorded_at from chart_initialized_at))
    ),

    account_currencies as (
        select distinct account_id, currency from cumulative_effective_balances
    ),

    account_currency_dates as (
        select account_id, currency, as_of_date
        from account_currencies
        cross join {{ ref("int_date_spine") }}
    ),

    as_of_candidates as (
        select
            account_currency_dates.as_of_date,
            account_currency_dates.account_id,
            account_currency_dates.currency,
            cumulative_effective_balances.effective,
            cumulative_effective_balances.version,
            cumulative_effective_balances.all_time_version,
            cumulative_effective_balances.`values`,
            row_number() over (
                partition by
                    account_currency_dates.account_id,
                    account_currency_dates.currency,
                    account_currency_dates.as_of_date
                order by
                    cumulative_effective_balances.effective desc,
                    cumulative_effective_balances.version desc,
                    cumulative_effective_balances.all_time_version desc
            ) as as_of_record_order_desc
        from account_currency_dates
        left join
            cumulative_effective_balances
            on cumulative_effective_balances.account_id
            = account_currency_dates.account_id
            and cumulative_effective_balances.currency = account_currency_dates.currency
            and cumulative_effective_balances.effective
            <= account_currency_dates.as_of_date
    ),

    final as (
        select
            as_of_date,
            cast(json_value(`values`, "$.settled.cr_balance") as numeric) as settled_cr,
            cast(json_value(`values`, "$.settled.dr_balance") as numeric) as settled_dr,
            account_id,
            currency
        from as_of_candidates
        where as_of_record_order_desc = 1 and effective is not null
    )

select *
from final
