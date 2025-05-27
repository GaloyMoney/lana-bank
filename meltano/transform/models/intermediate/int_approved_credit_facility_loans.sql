with approved_credit_facilities as (
    select
        *
    from {{ ref('int_credit_facility_events_combo') }}
    where approved
),

collateral_deposits as (
    select
        credit_facility_id,
        max(updated_recorded_at) as most_recent_collateral_deposit_at,
        any_value(collateral_new_amount_btc having max updated_recorded_at) as most_recent_collateral_deposit_amount_btc,
    from {{ ref('int_collateral_events') }}
    where action = "Add"
    group by credit_facility_id
),

disbursals as (
    select
        credit_facility_id,
        initialized_recorded_at as disbursal_initialized_recorded_at,
        concluded_recorded_at as disbursal_concluded_recorded_at,
        min(concluded_recorded_at) over(partition by credit_facility_id) as min_disbursal_concluded_recorded_at,
        disbursal_amount_usd as total_disbursed_usd,
        disbursal_amount_usd / sum(disbursal_amount_usd) over (partition by credit_facility_id) as disbursal_ratio,
        disbursal_id,
        obligation_id,
    from {{ ref('int_disbursal_events') }}
),

interest as (
    select
        credit_facility_id,
        sum(total_interest_posted_usd) as cf_total_interest_incurred_usd
    from {{ ref('int_interest_accrual_cycle_events') }}
    group by credit_facility_id
),

payments as (
    select
        credit_facility_id,
        sum(interest_amount_usd) as cf_total_interest_paid_usd,
        sum(disbursal_amount_usd) as cf_total_disbursal_paid_usd,
        max(if(interest_amount_usd > 0, payment_allocated_at, null)) as most_recent_interest_payment_timestamp,
        max(if(disbursal_amount_usd > 0, payment_allocated_at, null)) as most_recent_disbursal_payment_timestamp
    from {{ ref('int_payment_events') }}
    group by credit_facility_id
),

interest_paid_stats as (
  select
    credit_facility_id,
    disbursal_id,
    disbursal_ratio,
    cf_total_interest_incurred_usd,
    cf_total_interest_paid_usd,
    cf_total_disbursal_paid_usd,
    timestamp_diff(most_recent_interest_payment_timestamp, disbursal_concluded_recorded_at, day) as disbursal_interest_days,
    timestamp_diff(most_recent_interest_payment_timestamp, min_disbursal_concluded_recorded_at, day) as credit_facility_interest_days,
  from disbursals
  left join payments using (credit_facility_id)
  left join interest using (credit_facility_id)
),

interest_paid as (
  select
    credit_facility_id,
    disbursal_id,
    disbursal_interest_days,
    credit_facility_interest_days,
    disbursal_ratio * disbursal_interest_days as disbursal_weighted_interest_days,
    safe_divide(disbursal_ratio * disbursal_interest_days, sum(disbursal_ratio * disbursal_interest_days) over (partition by credit_facility_id)) as interest_paid_ratio,
    cf_total_interest_paid_usd * safe_divide(disbursal_ratio * disbursal_interest_days, sum(disbursal_ratio * disbursal_interest_days) over (partition by credit_facility_id)) as interest_paid_usd,
    cf_total_interest_incurred_usd * safe_divide(disbursal_ratio * disbursal_interest_days, sum(disbursal_ratio * disbursal_interest_days) over (partition by credit_facility_id)) as interest_incurred_usd,
    disbursal_ratio * cf_total_disbursal_paid_usd as disbursal_paid_usd,
  from interest_paid_stats
),

final as(
    select
        credit_facility_id,
        disbursal_id,
        initialized_recorded_at as facility_initialized_recorded_at,
        approved_recorded_at as facility_approved_recorded_at,
        activated_recorded_at as facility_activated_recorded_at,
        maturity_at as facility_maturity_at,
        activated_recorded_at as facility_start_date,
        maturity_at as facility_end_date,
        coalesce(facility_amount_usd, 0) as facility_amount_usd,
        annual_rate,
        one_time_fee_rate,
        initial_cvl,
        liquidation_cvl,
        margin_call_cvl,
        duration_value,
        duration_type,
        accrual_interval,
        accrual_cycle_interval,
        most_recent_interest_payment_timestamp,
        most_recent_disbursal_payment_timestamp,

        disbursal_initialized_recorded_at,
        disbursal_concluded_recorded_at,
        disbursal_concluded_recorded_at as disbursal_approved_recorded_at,
        disbursal_concluded_recorded_at as disbursal_start_date,
        maturity_at as disbursal_end_date,

        most_recent_collateral_deposit_at,
        cf_total_interest_incurred_usd,
        cf_total_interest_paid_usd,
        cf_total_disbursal_paid_usd,
        coalesce(collateral_amount_usd, 0) as cf_total_collateral_amount_usd,
        disbursal_ratio * coalesce(collateral_amount_usd, 0) as collateral_amount_usd,
        coalesce(total_disbursed_usd, 0) as total_disbursed_usd,

        disbursal_interest_days,
        credit_facility_interest_days,
        disbursal_weighted_interest_days,
        interest_incurred_usd,
        interest_paid_ratio,
        interest_paid_usd,
        disbursal_paid_usd,

        maturity_at < current_date() as matured,

        customer_id,
        facility_account_id,
        collateral_account_id,
        fee_income_account_id,
        interest_income_account_id,
        interest_defaulted_account_id,
        disbursed_defaulted_account_id,
        interest_receivable_due_account_id,
        disbursed_receivable_due_account_id,
        interest_receivable_overdue_account_id,
        disbursed_receivable_overdue_account_id,
        interest_receivable_not_yet_due_account_id,
        disbursed_receivable_not_yet_due_account_id,
        obligation_id,

    from approved_credit_facilities
         join disbursals using (credit_facility_id)
    left join collateral_deposits using (credit_facility_id)
    left join interest using (credit_facility_id)
    left join payments using (credit_facility_id)
    left join interest_paid using (credit_facility_id, disbursal_id)
)


select * from final
