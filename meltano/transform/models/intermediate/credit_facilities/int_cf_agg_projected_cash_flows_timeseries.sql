{{ config(materialized='table') }}

with projected_cash_flows_common as (
    select *
    from {{ ref('int_cf_projected_cash_flows_common') }}
    where credit_facility_end_date >= now_ts
),

grouped as (
    select
        now_ts,
        days_per_year,
        bench_mark_daily_interest_rate,
        days_from_now,
        sum(projected_disbursal_amount_in_cents)
            as projected_disbursal_amount_in_cents,
        sum(projected_payment_amount_in_cents)
            as projected_payment_amount_in_cents
    from projected_cash_flows_common
    group by
        now_ts,
        days_per_year,
        bench_mark_daily_interest_rate,
        days_from_now
    order by days_from_now
)

select
    *,
{% if target.type == 'bigquery' %}
    timestamp(
        timestamp_add(date(now_ts), interval cast(days_from_now as {{ dbt.type_int() }}) day)
    ) as date_from_now,
{% elif target.type == 'snowflake' %}
    TO_TIMESTAMP(
        TIMESTAMPADD(day, cast(days_from_now as {{ dbt.type_int() }}), date(now_ts))
    ) as date_from_now,
{% endif %}
    (projected_disbursal_amount_in_cents / 100.0)
        as projected_disbursal_amount_in_usd,
    (projected_payment_amount_in_cents / 100.0)
        as projected_payment_amount_in_usd
from grouped
