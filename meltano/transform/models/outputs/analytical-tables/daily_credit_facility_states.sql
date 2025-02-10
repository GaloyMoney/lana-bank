with approvals as (

    select
        id as credit_facility_id,
        date(recorded_at) as day,
        lax_bool(parsed_event.approved) as approved

    from {{ ref('stg_credit_facility_events') }}

    where event_type = "approval_process_concluded"


),

disbursal_inits as (

    select
        id as credit_facility_id,
        json_value(parsed_event.idx) as disbursal_idx,
        lax_int64(parsed_event.amount) as disbursal_amount

    from {{ ref('stg_credit_facility_events') }}

    where event_type = "disbursal_initiated"

),

disbursal_concludes as (

    select
        id as credit_facility_id,
        date(recorded_at) as day,
        json_value(parsed_event.idx) as disbursal_idx,
        lax_bool(parsed_event.canceled) as disbursal_canceled

    from {{ ref('stg_credit_facility_events') }}

    where event_type = "disbursal_concluded"

),

disbursals as (

    select
        credit_facility_id,
        day,
        sum(disbursal_amount) as disbursal_amount,
        count(distinct disbursal_idx) as n_disbursals,
        sum(
            case
                when disbursal_canceled then 0
                else disbursal_amount
            end
        ) as approved_disbursal_amount,
        countif(not disbursal_canceled) as approved_n_disbursals

    from disbursal_inits
    inner join disbursal_concludes using (credit_facility_id, disbursal_idx)

    group by credit_facility_id, day

),

payments as (

    select
        id as credit_facility_id,
        date(recorded_at) as day,
        sum(lax_int64(parsed_event.disbursal_amount)) as disbursal_amount_paid,
        sum(lax_int64(parsed_event.interest_amount)) as interest_amount_paid,
        count(*) as n_payments

    from {{ ref('stg_credit_facility_events') }}

    where event_type = "payment_recorded"

    group by credit_facility_id, day

),

interest as (

    select
        id as credit_facility_id,
        date(recorded_at) as day,
        lax_int64(parsed_event.amount) as interest_incurred

    from {{ ref('stg_credit_facility_events') }}

    where event_type = "interest_accrual_concluded"

),

collateral as (

    select
        id as credit_facility_id,
        date(recorded_at) as day,
        lax_int64(any_value(parsed_event.total_collateral having max recorded_at))
            as total_collateral

    from {{ ref('stg_credit_facility_events') }}

    where event_type = "collateral_updated"

    group by credit_facility_id, day

),

completions as (

    select distinct
        id as credit_facility_id,
        true as completed,
        date(recorded_at) as day

    from {{ ref('stg_credit_facility_events') }}

    where event_type = "completed"

),

joined as (

    select
        credit_facility_id,
        day,
        coalesce(approved, false) as approved,
        coalesce(disbursal_amount, 0) as disbursal_amount,
        coalesce(n_disbursals, 0) as n_disbursals,
        coalesce(approved_disbursal_amount, 0) as approved_disbursal_amount,
        coalesce(approved_n_disbursals, 0) as approved_n_disbursals,
        coalesce(disbursal_amount_paid, 0) as disbursal_amount_paid,
        coalesce(interest_amount_paid, 0) as interest_amount_paid,
        coalesce(n_payments, 0) as n_payments,
        coalesce(interest_incurred, 0) as interest_incurred,
        coalesce(
            last_value(total_collateral ignore nulls) over (
                partition by credit_facility_id
                order by day
            ), 0
        ) as total_collateral,
        coalesce(completed, false) as completed

    from approvals
    full join disbursals using (credit_facility_id, day)
    full join payments using (credit_facility_id, day)
    full join interest using (credit_facility_id, day)
    full join collateral using (credit_facility_id, day)
    full join completions using (credit_facility_id, day)

)

select
    joined.* except (approved, completed),
    last_value(approved ignore nulls) over (past)
    and not last_value(completed ignore nulls) over (past) as active,
    sum(approved_disbursal_amount) over (past) as total_disbursed,
    sum(approved_n_disbursals) over (past) as total_n_disbursals,
    sum(disbursal_amount_paid) over (past) as total_disbursal_amount_paid,
    sum(interest_amount_paid) over (past) as total_interest_amount_paid,
    sum(n_payments) over (past) as total_n_payments,
    sum(interest_incurred) over (past) as total_interest_incurred,
    total_collateral - lag(total_collateral, 1, 0) over (past) as collateral_change,
    last_value(close_price_usd ignore nulls) over (past) as close_price_usd

from joined
inner join {{ ref('days') }} using (day)

window
    past as (
        partition by credit_facility_id
        order by day
    )
