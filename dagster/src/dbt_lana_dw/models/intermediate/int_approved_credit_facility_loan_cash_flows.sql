with
    loans as (
        select
            *,
            disbursal_start_date as start_date,
            disbursal_end_date as end_date,
            'actual/360' as day_count_convention,
            annual_rate / 100.0 as annual_interest_rate

        from {{ ref("int_approved_credit_facility_loans") }}
    ),

    projections as (
        select
            *,
            safe_divide(
                annual_interest_rate,
                case
                    when ends_with(day_count_convention, '/360')
                    then 360.0
                    when ends_with(day_count_convention, '/365')
                    then 365.0
                    else
                        timestamp_diff(
                            timestamp(last_day(date(start_date), year)),
                            date_trunc(start_date, year),
                            day
                        )
                end
            ) as daily_interest_rate,
            case
                when ends_with(day_count_convention, '/360')
                then 360.0
                when ends_with(day_count_convention, '/365')
                then 365.0
                else
                    timestamp_diff(
                        timestamp(last_day(date(start_date), year)),
                        date_trunc(start_date, year),
                        day
                    )
            end as days_per_year,
            case
                when accrual_cycle_interval = 'end_of_day'
                then
                    generate_date_array(
                        date(start_date), last_day(date(end_date)), interval 1 day
                    )
                when accrual_cycle_interval = 'end_of_month'
                then
                    generate_date_array(
                        date(start_date), last_day(date(end_date)), interval 1 month
                    )
            end as interest_schedule_months
        from loans
    ),

    projected_interest_cash_flow_data as (
        select
            p.* except (interest_schedule_months),
            case
                when timestamp(date_trunc(projected_month, month)) < start_date
                then timestamp(date(start_date))
                else timestamp(date_trunc(projected_month, month))
            end as period_start_date,
            case
                when last_day(projected_month) > date(end_date)
                then timestamp(date(end_date))
                else timestamp(last_day(projected_month))
            end as period_end_date,
            'projected_interest_cash_flow' as cash_flow_type
        from projections as p, unnest(interest_schedule_months) as projected_month
    ),

    projected_principal_cash_flow_data as (
        select
            * except (interest_schedule_months),
            timestamp(date(start_date)) as period_start_date,
            timestamp(date(end_date)) as period_end_date,
            'projected_principal_cash_flow' as cash_flow_type
        from projections
    ),

    projected_cash_flow_data as (
        select *
        from projected_interest_cash_flow_data
        union all
        select *
        from projected_principal_cash_flow_data
    ),

    projected_time_data as (
        select
            *,
            timestamp_diff(date(period_end_date), date(period_start_date), day)
            + 1 as days_in_period
        from projected_cash_flow_data
    ),

    projected_cash_flows as (
        select
            *,
            case
                when cash_flow_type = 'projected_interest_cash_flow'
                then total_disbursed_usd * daily_interest_rate * days_in_period
                when cash_flow_type = 'projected_principal_cash_flow'
                then total_disbursed_usd
                else 0
            end as cash_flow_amount
        from projected_time_data
    ),

    final as (

        select
            credit_facility_id,
            disbursal_id,
            customer_id,
            facility_initialized_recorded_at,
            disbursal_initialized_recorded_at,
            disbursal_settled_recorded_at,
            start_date,
            end_date,
            duration_value,
            duration_type,
            annual_rate,
            accrual_interval,
            accrual_cycle_interval,
            facility_amount_usd,
            total_disbursed_usd,
            matured,
            day_count_convention,
            annual_interest_rate,
            daily_interest_rate,
            days_per_year,
            period_start_date,
            period_end_date,
            days_in_period,
            cash_flow_type,
            cash_flow_amount
        from projected_cash_flows
    )

select *
from final
