with aggregated as (

    select
        day,
        countif(active) as active_n_credit_facilities,
        sum(disbursal_amount_usd) as disbursal_amount_usd,
        sum(n_disbursals) as n_disbursals,
        sum(approved_disbursal_amount_usd) as approved_disbursal_amount_usd,
        sum(approved_n_disbursals) as approved_n_disbursals,
        sum(disbursal_amount_paid_usd) as disbursal_amount_paid_usd,
        sum(interest_amount_paid_usd) as interest_amount_paid_usd,
        sum(n_payments) as n_payments,
        sum(interest_incurred_usd) as interest_incurred_usd,
        sum(collateral_change_btc) as collateral_change_btc,
        any_value(close_price_usd_per_btc) as close_price_usd_per_btc,
        safe_divide(
            sum(initial_price_usd_per_btc * abs(collateral_change_btc)),
            sum(abs(collateral_change_btc))
        ) as initial_price_usd_per_btc


    from {{ ref('daily_credit_facility_states') }}

    group by day


),

avg_open_price as (

    select
        day,
        avg_open_prices[o] as collateral_avg_open_price

    from (

        select
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

        from aggregated

    ), unnest(days) as day with offset as o

)

select
    *,
    sum(collateral_change_btc) over (past) as total_collateral_btc,
    sum(approved_disbursal_amount_usd) over (past) as total_disbursed_usd,
    sum(approved_n_disbursals) over (past) as total_n_disbursals,
    sum(disbursal_amount_paid_usd) over (past) as total_disbursal_amount_paid_usd,
    sum(interest_amount_paid_usd) over (past) as total_interest_amount_paid_usd,
    sum(n_payments) over (past) as total_n_payments,
    sum(interest_incurred_usd) over (past) as total_interest_incurred_usd,

    sum(collateral_change_btc * initial_price_usd_per_btc)
        over (
            order by day
        )
        as initial_collateral_value_usd,
    sum(collateral_change_btc) over (past) * close_price_usd_per_btc as total_collateral_value_usd


from aggregated
inner join avg_open_price using (day)

window
    past as (
        order by day
    )
