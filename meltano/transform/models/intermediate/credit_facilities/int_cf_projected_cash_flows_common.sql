{{ config(materialized='table') }}

with terms_and_disbursal as (
    select
        cfe.event_id,
        cfe.recorded_at_date_key,
        cfe.recorded_at,
        cfe.event_type,
        cfe.terms_annual_rate,
        cfe.terms_duration_value,
        cfe.terms_initial_cvl,
        cfe.terms_liquidation_cvl,
        cfe.terms_margin_call_cvl,
        cfe.customer_id,
        cfe.terms_accrual_interval_type,
        cfe.terms_duration_type,
        cfe.terms_incurrence_interval_type,
        cfe.approval_process_started_recorded_at,
        cfe.approval_process_concluded_recorded_at,
        cfe.activated_recorded_at,
        cfe.activated_at,
        cfe.completed_recorded_at,
        cfe.completed_at,
        cfe.facility,
        cfe.approval_process_started_recorded_at_date_key,
        cfe.approval_process_concluded_recorded_at_date_key,
        cfe.approval_process_concluded_approved,
        cfe.activated_recorded_at_date_key,
        cfe.activated_at_date_key,
        cfe.completed_recorded_at_date_key,
        cfe.completed_at_date_key,
        d.idx,
        d.disbursal_concluded_event_recorded_at,
        d.amount,
        d.disbursal_concluded_event_recorded_at_date_key,
        facility as credit_facility_limit_in_cents,
        'actual/360' as credit_facility_day_count_convention,
        -- TODO get from proper source
        amount as disbursal_amount_in_cents,
        -- TODO get from proper source
        disbursal_concluded_event_recorded_at as disbursal_start_date,
        (terms_annual_rate / 100.0) as credit_facility_annual_interest_rate,
        5.53 / 100.0 as bench_mark_interest_rate,
    {% if target.type == 'bigquery' %}
        timestamp(current_date()) as now_ts,
        timestamp(date(activated_recorded_at)) as credit_facility_start_date,
    {% elif target.type == 'snowflake' %}
        TO_TIMESTAMP(current_date()) as now_ts,
        TO_TIMESTAMP(date(activated_recorded_at)) as credit_facility_start_date,
    {% endif %}
        case
            when terms_duration_type = 'months' then
            {% if target.type == 'bigquery' %}
                timestamp(
                    timestamp_add(
                        date(activated_recorded_at),
                        interval terms_duration_value month
                    )
            {% elif target.type == 'snowflake' %}
                TO_TIMESTAMP(
                    TIMESTAMPADD(
                        MONTH,
                        terms_duration_value,
                        DATE(activated_recorded_at)
                    )
            {% endif %}
                )
        end as credit_facility_end_date
    from {{ ref('int_credit_facilities') }} as cfe
    full join {{ ref('int_cf_disbursals') }} as d using (event_id)
    where
        disbursal_concluded_event_recorded_at_date_key != 19000101
        and terms_accrual_interval_type = 'end_of_month'
),

projections as (
    select
        *,
        (
            credit_facility_annual_interest_rate / nullif(
            case
            {% if target.type == 'bigquery' %}
                when
                    ends_with(credit_facility_day_count_convention, '/360')
                    then 360.0
                when
                    ends_with(credit_facility_day_count_convention, '/365')
                    then 365.0
                else
                    timestamp_diff(
                        timestamp(last_day(date(credit_facility_start_date), year)),
                        date_trunc(credit_facility_start_date, year),
                        day
                    )
            {% elif target.type == 'snowflake' %}
                when
                    ENDSWITH(credit_facility_day_count_convention, '/360')
                    then 360.0
                when
                    ENDSWITH(credit_facility_day_count_convention, '/365')
                    then 365.0
                else
                    TIMESTAMPDIFF(
                        day,
                        TO_TIMESTAMP(last_day(date(credit_facility_start_date), year)),
                        date_trunc(year, credit_facility_start_date)
                    )
            {% endif %}
            end, 0)
        ) as credit_facility_daily_interest_rate,
        (
            bench_mark_interest_rate / nullif(
            case
            {% if target.type == 'bigquery' %}
                when
                    ends_with(credit_facility_day_count_convention, '/360')
                    then 360.0
                when
                    ends_with(credit_facility_day_count_convention, '/365')
                    then 365.0
                else
                    timestamp_diff(
                        timestamp(last_day(date(credit_facility_start_date), year)),
                        date_trunc(credit_facility_start_date, year),
                        day
                    )
            {% elif target.type == 'snowflake' %}
                when
                    ENDSWITH(credit_facility_day_count_convention, '/360')
                    then 360.0
                when
                    ENDSWITH(credit_facility_day_count_convention, '/365')
                    then 365.0
                else
                    TIMESTAMPDIFF(
                        day,
                        TO_TIMESTAMP(last_day(date(credit_facility_start_date), year)),
                        date_trunc(year, credit_facility_start_date)
                    )
            {% endif %}
            end, 0)
        ) as bench_mark_daily_interest_rate,
        case
        {% if target.type == 'bigquery' %}
            when
                ends_with(credit_facility_day_count_convention, '/360')
                then 360.0
            when
                ends_with(credit_facility_day_count_convention, '/365')
                then 365.0
            else
                timestamp_diff(
                    timestamp(last_day(date(credit_facility_start_date), year)),
                    date_trunc(credit_facility_start_date, year),
                    day
                )
        {% elif target.type == 'snowflake' %}
            when
                ENDSWITH(credit_facility_day_count_convention, '/360')
                then 360.0
            when
                ENDSWITH(credit_facility_day_count_convention, '/365')
                then 365.0
            else
                TIMESTAMPDIFF(
                    day,
                    TO_TIMESTAMP(last_day(date(credit_facility_start_date), year)),
                    date_trunc(year, credit_facility_start_date)
                )
        {% endif %}
        end as days_per_year,
        (bench_mark_interest_rate / nullif(credit_facility_annual_interest_rate, 0)) as breakeven_disbursal_percent,
        (credit_facility_limit_in_cents * (bench_mark_interest_rate / nullif(credit_facility_annual_interest_rate, 0))) as breakeven_disbursal_amount_in_cents,
        case
        {% if target.type == 'bigquery' %}
            when terms_accrual_interval_type = 'end_of_day' then
                    generate_date_array(
                        date(disbursal_start_date),
                        last_day(date(credit_facility_end_date)),
                        interval 1 day
                    )
            when terms_accrual_interval_type = 'end_of_month' then
                generate_date_array(
                    date(disbursal_start_date),
                    last_day(date(credit_facility_end_date)),
                    interval 1 month
                )
        {% elif target.type == 'snowflake' %}
            when terms_accrual_interval_type = 'end_of_day' then
                (SELECT
                    ARRAY_AGG("DAY") A
                FROM (
                    SELECT ROW_NUMBER() OVER (ORDER BY NULL) - 1 "DAY"
                    FROM TABLE(GENERATOR(ROWCOUNT => 1000))
                )
                WHERE "DAY" <= CEIL(DATEDIFF(DAYS, date(disbursal_start_date), last_day(date(credit_facility_end_date)))))
            when terms_accrual_interval_type = 'end_of_month' then
                (SELECT
                    ARRAY_AGG("MONTH") A
                FROM (
                    SELECT ROW_NUMBER() OVER (ORDER BY NULL) - 1 "MONTH"
                    FROM TABLE(GENERATOR(ROWCOUNT => 1000))
                )
                WHERE "MONTH" <= CEIL(DATEDIFF(MONTHS, date(disbursal_start_date), last_day(date(credit_facility_end_date)))))
        {% endif %}
        end as interest_schedule_months
    from terms_and_disbursal
),

projected_interest_payment_data as (
    select
        p.event_id,
        p.recorded_at_date_key,
        p.recorded_at,
        p.event_type,
        p.terms_annual_rate,
        p.terms_duration_value,
        p.terms_initial_cvl,
        p.terms_liquidation_cvl,
        p.terms_margin_call_cvl,
        p.customer_id,
        p.terms_accrual_interval_type,
        p.terms_duration_type,
        p.terms_incurrence_interval_type,
        p.approval_process_started_recorded_at,
        p.approval_process_concluded_recorded_at,
        p.activated_recorded_at,
        p.activated_at,
        p.completed_recorded_at,
        p.completed_at,
        p.facility,
        p.approval_process_started_recorded_at_date_key,
        p.approval_process_concluded_recorded_at_date_key,
        p.approval_process_concluded_approved,
        p.activated_recorded_at_date_key,
        p.activated_at_date_key,
        p.completed_recorded_at_date_key,
        p.completed_at_date_key,
        p.idx,
        p.disbursal_concluded_event_recorded_at,
        p.amount,
        p.disbursal_concluded_event_recorded_at_date_key,
        p.credit_facility_limit_in_cents,
        p.credit_facility_day_count_convention,
        p.disbursal_amount_in_cents,
        p.disbursal_start_date,
        p.credit_facility_annual_interest_rate,
        p.bench_mark_interest_rate,
        p.now_ts,
        p.credit_facility_start_date,
        p.credit_facility_end_date,
        p.credit_facility_daily_interest_rate,
        p.bench_mark_daily_interest_rate,
        p.days_per_year,
        p.breakeven_disbursal_percent,
        p.breakeven_disbursal_amount_in_cents,
        case
            when
        {% if target.type == 'bigquery' %}
                timestamp(date_trunc(projected_month, month))
                < disbursal_start_date
                then
                    timestamp(date(disbursal_start_date))
            else
                timestamp(date_trunc(projected_month, month))
        {% elif target.type == 'snowflake' %}
                TO_TIMESTAMP(DATE_TRUNC(MONTH, DATEADD(MONTH, projected_month.value, date(disbursal_start_date))))
                < disbursal_start_date
                then
                    TO_TIMESTAMP(date(disbursal_start_date))
            else
                TO_TIMESTAMP(DATE_TRUNC(MONTH, DATEADD(MONTH, projected_month.value, date(disbursal_start_date))))
        {% endif %}
        end as period_start_date,
        case
        {% if target.type == 'bigquery' %}
            when last_day(projected_month) > date(credit_facility_end_date)
                then
                    timestamp(date(credit_facility_end_date))
            else
                timestamp(last_day(projected_month))
        {% elif target.type == 'snowflake' %}
            when last_day(DATEADD(MONTH, projected_month.value, date(disbursal_start_date))) > date(credit_facility_end_date)
                then
                    TO_TIMESTAMP(date(credit_facility_end_date))
            else
                TO_TIMESTAMP(last_day(DATEADD(MONTH, projected_month.value, date(disbursal_start_date))))
        {% endif %}
        end as period_end_date,
        'projected_interest_payment' as payment_type
    from projections as p,
        {% if target.type == 'bigquery' %}
            unnest(interest_schedule_months) as projected_month
        {% elif target.type == 'snowflake' %}
            table(flatten(input => interest_schedule_months, mode => 'array')) as projected_month
        {% endif %}
),

projected_principal_payment_data as (
    select
        p.event_id,
        p.recorded_at_date_key,
        p.recorded_at,
        p.event_type,
        p.terms_annual_rate,
        p.terms_duration_value,
        p.terms_initial_cvl,
        p.terms_liquidation_cvl,
        p.terms_margin_call_cvl,
        p.customer_id,
        p.terms_accrual_interval_type,
        p.terms_duration_type,
        p.terms_incurrence_interval_type,
        p.approval_process_started_recorded_at,
        p.approval_process_concluded_recorded_at,
        p.activated_recorded_at,
        p.activated_at,
        p.completed_recorded_at,
        p.completed_at,
        p.facility,
        p.approval_process_started_recorded_at_date_key,
        p.approval_process_concluded_recorded_at_date_key,
        p.approval_process_concluded_approved,
        p.activated_recorded_at_date_key,
        p.activated_at_date_key,
        p.completed_recorded_at_date_key,
        p.completed_at_date_key,
        p.idx,
        p.disbursal_concluded_event_recorded_at,
        p.amount,
        p.disbursal_concluded_event_recorded_at_date_key,
        p.credit_facility_limit_in_cents,
        p.credit_facility_day_count_convention,
        p.disbursal_amount_in_cents,
        p.disbursal_start_date,
        p.credit_facility_annual_interest_rate,
        p.bench_mark_interest_rate,
        p.now_ts,
        p.credit_facility_start_date,
        p.credit_facility_end_date,
        p.credit_facility_daily_interest_rate,
        p.bench_mark_daily_interest_rate,
        p.days_per_year,
        p.breakeven_disbursal_percent,
        p.breakeven_disbursal_amount_in_cents,
    {% if target.type == 'bigquery' %}
        timestamp(date(disbursal_start_date)) as period_start_date,
        timestamp(date(credit_facility_end_date)) as period_end_date,
    {% elif target.type == 'snowflake' %}
        TO_TIMESTAMP(date(disbursal_start_date)) as period_start_date,
        TO_TIMESTAMP(date(credit_facility_end_date)) as period_end_date,
    {% endif %}
        'projected_principal_payment' as payment_type
    from projections as p
),

projected_disbursal_data as (
    select
        p.event_id,
        p.recorded_at_date_key,
        p.recorded_at,
        p.event_type,
        p.terms_annual_rate,
        p.terms_duration_value,
        p.terms_initial_cvl,
        p.terms_liquidation_cvl,
        p.terms_margin_call_cvl,
        p.customer_id,
        p.terms_accrual_interval_type,
        p.terms_duration_type,
        p.terms_incurrence_interval_type,
        p.approval_process_started_recorded_at,
        p.approval_process_concluded_recorded_at,
        p.activated_recorded_at,
        p.activated_at,
        p.completed_recorded_at,
        p.completed_at,
        p.facility,
        p.approval_process_started_recorded_at_date_key,
        p.approval_process_concluded_recorded_at_date_key,
        p.approval_process_concluded_approved,
        p.activated_recorded_at_date_key,
        p.activated_at_date_key,
        p.completed_recorded_at_date_key,
        p.completed_at_date_key,
        p.idx,
        p.disbursal_concluded_event_recorded_at,
        p.amount,
        p.disbursal_concluded_event_recorded_at_date_key,
        p.credit_facility_limit_in_cents,
        p.credit_facility_day_count_convention,
        p.disbursal_amount_in_cents,
        p.disbursal_start_date,
        p.credit_facility_annual_interest_rate,
        p.bench_mark_interest_rate,
        p.now_ts,
        p.credit_facility_start_date,
        p.credit_facility_end_date,
        p.credit_facility_daily_interest_rate,
        p.bench_mark_daily_interest_rate,
        p.days_per_year,
        p.breakeven_disbursal_percent,
        p.breakeven_disbursal_amount_in_cents,
    {% if target.type == 'bigquery' %}
        timestamp(date(now_ts)) as period_start_date,
        timestamp(
        timestamp_add(date(disbursal_start_date), interval -1 day)
    {% elif target.type == 'snowflake' %}
        TO_TIMESTAMP(date(now_ts)) as period_start_date,
        TO_TIMESTAMP(
        TIMESTAMPADD(DAY, -1, DATE(disbursal_start_date))
    {% endif %}
        ) as period_end_date,
        'projected_disbursal' as payment_type
    from projections as p
),

projected_payment_data as (
    select * from projected_interest_payment_data
    union all
    select * from projected_principal_payment_data
    union all
    select * from projected_disbursal_data
),

projected_time_data as (
    select
        *,
    {% if target.type == 'bigquery' %}
        cast(timestamp_diff(date(period_end_date), date(now_ts), day) + 1 as {{ dbt.type_float() }}) as days_from_now,
        timestamp_diff(date(period_end_date), date(period_start_date), day) + 1 as days_in_the_period
    {% elif target.type == 'snowflake' %}
        cast(TIMESTAMPDIFF(day ,date(period_end_date), date(now_ts)) + 1 as {{ dbt.type_float() }}) as days_from_now,
        TIMESTAMPDIFF(day, date(period_end_date), date(period_start_date)) + 1 as days_in_the_period
    {% endif %}
    from projected_payment_data
),

projected_cash_flows_common as (
    select
        customer_id,
        event_id,
        idx as disbursal_idx,
        credit_facility_start_date,
        credit_facility_end_date,
        bench_mark_interest_rate,
        bench_mark_daily_interest_rate,
        credit_facility_annual_interest_rate,
        credit_facility_daily_interest_rate,
        now_ts,
        days_per_year,
        days_in_the_period,
        days_from_now,
        case
            when payment_type = 'projected_disbursal'
                then cast(-disbursal_amount_in_cents as {{ dbt.type_float() }})
            else 0
        end as projected_disbursal_amount_in_cents,
        case
            when payment_type = 'projected_interest_payment'
                then disbursal_amount_in_cents * credit_facility_daily_interest_rate * days_in_the_period
            when payment_type = 'projected_principal_payment'
                then disbursal_amount_in_cents
            else 0
        end as projected_payment_amount_in_cents
    from projected_time_data
)

select *
from projected_cash_flows_common
