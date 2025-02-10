select
    day,
    countif(active) as active_n_credit_facilities,
    sum(disbursal_amount) as disbursal_amount,
    sum(n_disbursals) as n_disbursals,
    sum(approved_disbursal_amount) as approved_disbursal_amount,
    sum(approved_n_disbursals) as approved_n_disbursals,
    sum(disbursal_amount_paid) as disbursal_amount_paid,
    sum(interest_amount_paid) as interest_amount_paid,
    sum(n_payments) as n_payments,
    sum(interest_incurred) as interest_incurred,
    sum(total_collateral) as total_collateral,
    sum(total_disbursed) as total_disbursed,
    sum(total_n_disbursals) as total_n_disbursals,
    sum(total_disbursal_amount_paid) as total_disbursal_amount_paid,
    sum(total_interest_amount_paid) as total_interest_amount_paid,
    sum(total_n_payments) as total_n_payments,
    sum(total_interest_incurred) as total_interest_incurred,
    sum(collateral_change) as collateral_change,
    any_value(close_price_usd) as close_price_usd

from {{ ref('daily_credit_facility_states') }}

group by day
