with approved as (
    select
        *
    from {{ ref('int_credit_facility_events_combo') }}
    where approved
),

payments as (
    select
        credit_facility_id,
        sum(interest_amount) as total_interest_paid,
        sum(disbursal_amount) as total_disbursement_paid,
        max(if(interest_amount > 0, payment_allocated_at, null)) as most_recent_interest_payment_timestamp,
        max(if(disbursal_amount > 0, payment_allocated_at, null)) as most_recent_disbursement_payment_timestamp
    from {{ ref('int_payment_events') }}
    group by credit_facility_id
),

interest as (
    select
        credit_facility_id,
        sum(posted_total) as total_interest_incurred
    from {{ ref('int_interest_accrual_cycle_events') }}
    group by credit_facility_id
),

collateral_deposits as (

    select
        credit_facility_id,
        max(updated_recorded_at) as most_recent_collateral_deposit_at,
        any_value(new_value having max updated_recorded_at) as most_recent_collateral_deposit_amount,

    from {{ ref('int_collateral_events') }}
    where action = "Add"
    group by credit_facility_id

),

disbursements as (

    select
        credit_facility_id,
        sum(initialized_amount) as total_disbursed
    from {{ ref('int_disbursal_events') }}
    where approved
    group by credit_facility_id

)


select
    credit_facility_id,
    initialized_recorded_at as initialized_at,
    initialized_recorded_at,
    approved_recorded_at,
    activated_recorded_at,
    maturity_at,
    maturity_at as end_date,
    coalesce(facility_amount, 0) as facility,
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
    most_recent_disbursement_payment_timestamp,
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
    most_recent_collateral_deposit_at,
    row_number() over () as credit_facility_key,
    total_interest_paid,
    total_disbursement_paid,
    total_interest_incurred,
    coalesce(collateral, 0) as total_collateral,
    total_disbursed,
    maturity_at < current_date() as matured

from approved
left join payments using (credit_facility_id)
left join interest using (credit_facility_id)
left join collateral_deposits using (credit_facility_id)
left join disbursements using (credit_facility_id)
