{{ config(materialized='table') }}

{% if target.type == 'bigquery' %}
{% set target_database = '' %}
{% elif target.type == 'snowflake' %}
{% set target_database = target.database + '.' %}
{% endif %}

with projected_cash_flows_common as (
    select *
    from {{ ref('int_cf_projected_cash_flows_common') }}
),

grouped as (
    select
        customer_id,
        event_id,
        credit_facility_start_date,
        credit_facility_end_date,
        now_ts,
        days_per_year,
        bench_mark_daily_interest_rate,
        days_from_now,
        sum(projected_disbursal_amount_in_cents)
            as projected_disbursal_amount_in_cents,
        case
            when days_from_now < 0 then 0 else
                sum(projected_payment_amount_in_cents)
        end as projected_payment_amount_in_cents
    from projected_cash_flows_common
    group by
        customer_id,
        event_id,
        credit_facility_start_date,
        credit_facility_end_date,
        now_ts,
        days_per_year,
        bench_mark_daily_interest_rate,
        days_from_now
    order by days_from_now
),

arrayed as (
    select
        customer_id,
        event_id,
        credit_facility_start_date,
        credit_facility_end_date,
        now_ts,
        days_per_year,
        bench_mark_daily_interest_rate,
        array_agg(projected_disbursal_amount_in_cents)
            as projected_disbursal_amount_in_cents,
        array_agg(days_from_now) as days_from_now,
        array_agg(projected_payment_amount_in_cents) as cash_flows
    from grouped
    group by
        customer_id,
        event_id,
        credit_facility_start_date,
        credit_facility_end_date,
        now_ts,
        days_per_year,
        bench_mark_daily_interest_rate
),

with_risk as (
    select
        customer_id,
        event_id,
        credit_facility_start_date,
        credit_facility_end_date,
        now_ts,
        days_per_year,
        bench_mark_daily_interest_rate,
        projected_disbursal_amount_in_cents,
        days_from_now,
        cash_flows,
        {{ target_database }}{{ target.schema }}.udf_loan_pv(
            bench_mark_daily_interest_rate,
            days_from_now,
            projected_disbursal_amount_in_cents
        ) as disbursal_pv,
        {{ target_database }}{{ target.schema }}.udf_loan_pv(
            bench_mark_daily_interest_rate, days_from_now, cash_flows
        ) as pv,
        (
            {{ target_database }}{{ target.schema }}.udf_loan_ytm(
                bench_mark_daily_interest_rate, days_from_now, cash_flows
            ) * 365.0
        ) as ytm,
        {{ target_database }}{{ target.schema }}.udf_loan_mac_duration(
            bench_mark_daily_interest_rate, days_from_now, cash_flows
        ) as mac_duration,
        (
            {{ target_database }}{{ target.schema }}.udf_loan_mod_duration(
                bench_mark_daily_interest_rate, days_from_now, cash_flows
            ) / 365.0
        ) as mod_duration,
        (
            {{ target_database }}{{ target.schema }}.udf_loan_convexity(
                bench_mark_daily_interest_rate, days_from_now, cash_flows
            ) / (365.0 * 365.0)
        ) as convexity,
        {{ target_database }}{{ target.schema }}.udf_loan_pv_delta_on_interest_rate_delta_with_convex(
            bench_mark_daily_interest_rate,
            days_from_now,
            cash_flows,
            0.0001 / nullif(days_per_year, 0)
        ) as dv01,
        {{ target_database }}{{ target.schema }}.udf_loan_pv(
            bench_mark_daily_interest_rate + (0.0001 / nullif(days_per_year, 0)),
            days_from_now,
            cash_flows
        ) as pv_at_dv01
    from arrayed
),

final as (
    select
        customer_id,
        event_id,
        credit_facility_start_date,
        credit_facility_end_date,
        now_ts,
        days_per_year,
        bench_mark_daily_interest_rate,
        projected_disbursal_amount_in_cents,
        days_from_now,
        cash_flows,
        (disbursal_pv / 100.0) as disbursal_pv,
        (pv / 100.0) as pv,
        (
            (
                {{ target_database }}{{ target.schema }}.udf_loan_pv(
                    bench_mark_daily_interest_rate, days_from_now, cash_flows
                ) + disbursal_pv
            ) / 100.0
        ) as npv,
        ytm,
        (
            {{ target_database }}{{ target.schema }}.udf_loan_ytm_from_price(
                -(disbursal_pv), days_from_now, cash_flows
            ) * 365.0
        ) as ytm_from_price,
        mac_duration,
        case
        {% if target.type == 'bigquery' %}
            when is_nan(mac_duration)
                then timestamp('1900-01-01')
            else
                timestamp(
                    timestamp_add(
                        date(now_ts), interval cast(mac_duration as {{ dbt.type_int() }}) day
                    )
                )
        {% elif target.type == 'snowflake' %}
            when try_cast(mac_duration::varchar as float) = 'NaN'
                then TO_TIMESTAMP('1900-01-01')
            else
                TO_TIMESTAMP(
                    TIMESTAMPADD(
                        day, cast(mac_duration as {{ dbt.type_int() }}), date(now_ts)
                    )
                )
        {% endif %}
        end as mac_duration_date,
        (dv01 / 100.0) as dv01,
        (pv_at_dv01 / 100.0) as pv_at_dv01
    from with_risk
)

select * from final
