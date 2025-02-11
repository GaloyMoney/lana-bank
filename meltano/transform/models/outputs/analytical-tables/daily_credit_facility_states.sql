-- TODO: these daily states should be derived from an instantaneous state table
-- this would reduce any potential inconsistencies due to the aggregation.
-- It can be done in a future PR if it turns out the interface works.
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
        lax_int64(parsed_event.amount) / 100 as disbursal_amount_usd

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
        sum(disbursal_amount_usd) as disbursal_amount_usd,
        count(distinct disbursal_idx) as n_disbursals,
        sum(
            case
                when disbursal_canceled then 0
                else disbursal_amount_usd
            end
        ) as approved_disbursal_amount_usd,
        countif(not disbursal_canceled) as approved_n_disbursals

    from disbursal_inits
    inner join disbursal_concludes using (credit_facility_id, disbursal_idx)

    group by credit_facility_id, day

),

payments as (

    select
        id as credit_facility_id,
        date(recorded_at) as day,
        sum(lax_int64(parsed_event.disbursal_amount)) / 100 as disbursal_amount_paid_usd,
        sum(lax_int64(parsed_event.interest_amount)) / 100 as interest_amount_paid_usd,
        count(*) as n_payments

    from {{ ref('stg_credit_facility_events') }}

    where event_type = "payment_recorded"

    group by credit_facility_id, day

),

interest as (

    select
        id as credit_facility_id,
        date(recorded_at) as day,
        lax_int64(parsed_event.amount) / 100 as interest_incurred_usd

    from {{ ref('stg_credit_facility_events') }}

    where event_type = "interest_accrual_concluded"

),

collateral_updates as (

    select
        id as credit_facility_id,
        recorded_at,
        lax_int64(parsed_event.total_collateral)
        / {{ var('sats_per_bitcoin') }}
            as total_collateral_btc,
        json_value(parsed_event.audit_info.audit_entry_id) as audit_entry_id

    from {{ ref('stg_credit_facility_events') }}

    where event_type = "collateral_updated"

),

collateralization as (

    select
        id as credit_facility_id,
        lax_int64(parsed_event.price) / 100 as initial_price_usd_per_btc,
        json_value(parsed_event.audit_info.audit_entry_id) as audit_entry_id

    from {{ ref('stg_credit_facility_events') }}

    where event_type = "collateralization_changed"

),

collateral as (

    select
        credit_facility_id,
        date(recorded_at) as day,
        any_value(initial_price_usd_per_btc) as initial_price_usd_per_btc,
        any_value(total_collateral_btc having max recorded_at) as total_collateral_btc

    from collateral_updates
    left join collateralization using (credit_facility_id, audit_entry_id)

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
        initial_price_usd_per_btc,
        approved,
        coalesce(
            last_value(close_price_usd_per_btc ignore nulls) over (
                order by day asc
            ),
            last_value(close_price_usd_per_btc ignore nulls) over (
                order by day desc
            )
        ) as close_price_usd_per_btc,
        coalesce(disbursal_amount_usd, 0) as disbursal_amount_usd,
        coalesce(n_disbursals, 0) as n_disbursals,
        coalesce(approved_disbursal_amount_usd, 0) as approved_disbursal_amount_usd,
        coalesce(approved_n_disbursals, 0) as approved_n_disbursals,
        coalesce(disbursal_amount_paid_usd, 0) as disbursal_amount_paid_usd,
        coalesce(interest_amount_paid_usd, 0) as interest_amount_paid_usd,
        coalesce(n_payments, 0) as n_payments,
        coalesce(interest_incurred_usd, 0) as interest_incurred_usd,
        coalesce(
            last_value(total_collateral_btc ignore nulls) over (
                partition by credit_facility_id
                order by day
            ), 0
        ) as total_collateral_btc,
        coalesce(completed, false) as completed

    from {{ ref('days') }}
    full join approvals using (day)
    full join disbursals using (credit_facility_id, day)
    full join payments using (credit_facility_id, day)
    full join interest using (credit_facility_id, day)
    full join collateral using (credit_facility_id, day)
    full join completions using (credit_facility_id, day)

),

filled as (

    select
        joined.* except (initial_price_usd_per_btc, approved, completed),
        coalesce(initial_price_usd_per_btc, close_price_usd_per_btc) as initial_price_usd_per_btc,
        last_value(approved ignore nulls) over (past)
        and not last_value(completed ignore nulls) over (past) as active,
        sum(approved_disbursal_amount_usd) over (past) as total_disbursed_usd,
        sum(approved_n_disbursals) over (past) as total_n_disbursals,
        sum(disbursal_amount_paid_usd) over (past) as total_disbursal_amount_paid_usd,
        sum(interest_amount_paid_usd) over (past) as total_interest_amount_paid_usd,
        sum(n_payments) over (past) as total_n_payments,
        sum(interest_incurred_usd) over (past) as total_interest_incurred_usd,
        total_collateral_btc - lag(total_collateral_btc, 1, 0) over (past) as collateral_change_btc

    from joined

    window
        past as (
            partition by credit_facility_id
            order by day
        )

),

avg_open_price as (

    select
        credit_facility_id,
        day,
        avg_open_prices[o] as collateral_avg_open_price

    from (

        select
            credit_facility_id,
            array_agg(
                day
                order by day
            ) as days,
            {{ target.schema }}.udf_avg_open_price(
                array_agg(
                    collateral_change_btc
                    order by day
                ),
                array_agg(
                    initial_price_usd_per_btc
                    order by day
                )
            ) as avg_open_prices

        from filled

        group by credit_facility_id

    ), unnest(days) as day with offset as o

)

select
    *,
    sum(collateral_change_btc * initial_price_usd_per_btc)
        over (
            partition by credit_facility_id
            order by day
        )
        as initial_collateral_value_usd,
    total_collateral_btc * close_price_usd_per_btc as total_collateral_value_usd

from filled
inner join avg_open_price using (credit_facility_id, day)
