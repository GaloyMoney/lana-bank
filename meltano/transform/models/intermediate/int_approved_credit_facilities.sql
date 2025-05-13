with approved as (
    select
        *
    from {{ ref('int_credit_facility_events_combo') }}
    where approved
),

payments as (

    select
        id as credit_facility_id,
        sum(cast(json_value(event, "$.interest_amount") as numeric))
            as total_interest_paid,
        sum(cast(json_value(event, "$.disbursal_amount") as numeric))
            as total_disbursement_paid,
        max(
            if(
                coalesce(
                    cast(json_value(event, "$.interest_amount") as numeric), 0
                )
                > 0,
                recorded_at,
                null
            )
        ) as most_recent_interest_payment_timestamp,
        max(
            if(
                coalesce(
                    cast(json_value(event, "$.disbursal_amount") as numeric),
                    0
                )
                > 0,
                recorded_at,
                null
            )
        ) as most_recent_disbursement_payment_timestamp

    from {{ ref('stg_credit_facility_events') }}

    where event_type = "payment_recorded"

    group by credit_facility_id

),

interest as (

    select
        id as credit_facility_id,
        sum(cast(json_value(event, "$.amount") as numeric))
            as total_interest_incurred

    from {{ ref('stg_credit_facility_events') }}

    where event_type = "interest_accrual_concluded"

    group by credit_facility_id

),

collateral_deposits as (

    select
        id as credit_facility_id,
        parse_timestamp(
            "%Y-%m-%dT%H:%M:%E6SZ",
            json_value(
                any_value(event having max recorded_at),
                "$.recorded_at"
            ),
            "UTC"
        ) as most_recent_collateral_deposit

    from {{ ref('stg_credit_facility_events') }}

    where
        event_type = "collateral_updated"
        and json_value(event, "$.action") = "Add"

    group by credit_facility_id

),

disbursements as (

    select
        id as credit_facility_id,
        sum(cast(json_value(event, "$.amount") as numeric)) as total_disbursed

    from {{ ref('stg_credit_facility_events') }}

    where event_type = "disbursal_initiated"

    group by credit_facility_id

)

select
    credit_facility_id,
    initialized_recorded_at as initialized_at,
    maturity_at as end_date,
    accrual_interval,
    accrual_cycle_interval,
    cast(null as timestamp) as most_recent_interest_payment_timestamp,
    cast(null as timestamp) as most_recent_disbursement_payment_timestamp,
    annual_rate,
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
    cast(null as date) most_recent_collateral_deposit,
    row_number() over () as credit_facility_key,
    coalesce(facility_amount, 0) as facility,
    coalesce(null, 0) as total_interest_paid,
    coalesce(null, 0) as total_disbursement_paid,
    coalesce(null, 0) as total_interest_incurred,
    coalesce(collateral, 0) as total_collateral,
    coalesce(null, 0) as total_disbursed,
    maturity_at < current_date() as matured

from approved
left join payments using (credit_facility_id)
left join interest using (credit_facility_id)
left join collateral_deposits using (credit_facility_id)
left join disbursements using (credit_facility_id)
