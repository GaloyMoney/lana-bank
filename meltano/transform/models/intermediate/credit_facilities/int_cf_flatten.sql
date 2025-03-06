{{ config(materialized='table') }}

with credit_facilities as (

    select * from {{ ref('int_credit_facilities') }}

),

int_cf_disbursals as (

    select
        event_id,
        max(recorded_at_date_key) as disbursal_recorded_at_date_key,
        max(recorded_at) as disbursal_recorded_at,
        max(disbursal_concluded_event_recorded_at_date_key) as disbursal_concluded_event_recorded_at_date_key,
        max(disbursal_concluded_event_recorded_at) as disbursal_concluded_event_recorded_at,
        sum(amount) as total_disbursed_amount
    from {{ ref('int_cf_disbursals') }}
    group by event_id

),

int_cf_collaterals as (

    select
        event_id,
        max(recorded_at_date_key) as collateral_recorded_at_date_key,
        max(recorded_at) as collateral_recorded_at,

        max(recorded_in_ledger_at_date_key) as recorded_in_ledger_at_date_key,
        max(recorded_in_ledger_at) as recorded_in_ledger_at,
        max(collateralization_changed_event_recorded_at_date_key)
            as collateralization_changed_event_recorded_at_date_key,
        max(collateralization_changed_event_recorded_at)
            as collateralization_changed_event_recorded_at,

        {% if target.type == 'bigquery' %}
            array_agg(
                collateralization_changed_state
                order by collateralization_changed_event_recorded_at desc limit 1
            )[safe_ordinal(1)] as collateralization_changed_state,

            sum(diff) as total_collateral_summed,
            array_agg(
                total_collateral
                order by recorded_at desc limit 1)[
                safe_ordinal(1)
            ] as total_collateral,

            sum(outstanding_disbursed) as outstanding_disbursed,
            sum(outstanding_interest) as outstanding_interest,

            ((sum((diff * price)) / nullif(sum(diff), 0)) / 100.0) as average_initial_price_usd,
            array_agg(
                initial_collateral_value_usd
                order by recorded_at desc limit 1)[
                safe_ordinal(1)
            ] as initial_collateral_value_usd,
            array_agg(
                total_collateral_value_usd
                order by recorded_at desc limit 1)[
                safe_ordinal(1)
            ] as total_collateral_value_usd,
            array_agg(
                last_btc_price_usd
                order by recorded_at desc limit 1)[
                safe_ordinal(1)
            ] as last_btc_price_usd
        {% elif target.type == 'snowflake' %}
            GET(array_agg(collateralization_changed_state) WITHIN GROUP(order by collateralization_changed_event_recorded_at desc), 0) as collateralization_changed_state,

            sum(diff) as total_collateral_summed,
            GET(array_agg(total_collateral) WITHIN GROUP(order by recorded_at desc), 0) as total_collateral,

            sum(outstanding_disbursed) as outstanding_disbursed,
            sum(outstanding_interest) as outstanding_interest,

            ((sum((diff * price)) / nullif(sum(diff), 0)) / 100.0) as average_initial_price_usd,
            GET(array_agg(initial_collateral_value_usd) WITHIN GROUP(order by recorded_at desc), 0) as initial_collateral_value_usd,
            GET(array_agg(total_collateral_value_usd) WITHIN GROUP(order by recorded_at desc), 0) as total_collateral_value_usd,
            GET(array_agg(last_btc_price_usd) WITHIN GROUP (order by recorded_at desc), 0) as last_btc_price_usd
        {% endif %}

    from {{ ref('int_cf_collaterals') }}
    group by event_id

),

int_cf_payments as (

    select
        event_id,
        max(recorded_at_date_key) as payment_recorded_at_date_key,
        max(recorded_at) as payment_recorded_at,
        max(recorded_in_ledger_at_date_key) as payment_recorded_in_ledger_at_date_key,
        max(recorded_in_ledger_at) as payment_recorded_in_ledger_at,
        sum(disbursal_amount) as disbursal_amount,
        sum(interest_amount) as interest_amount
    from {{ ref('int_cf_payments') }}
    group by event_id

)

select
    cfe.*,

    d.disbursal_recorded_at_date_key,
    d.disbursal_recorded_at,
    d.disbursal_concluded_event_recorded_at_date_key,
    d.disbursal_concluded_event_recorded_at,
    d.total_disbursed_amount,

    c.collateral_recorded_at_date_key,
    c.collateral_recorded_at,
    c.recorded_in_ledger_at_date_key,
    c.recorded_in_ledger_at,
    c.collateralization_changed_event_recorded_at_date_key,
    c.collateralization_changed_event_recorded_at,
    c.collateralization_changed_state,
    c.total_collateral_summed,
    c.total_collateral,
    c.outstanding_disbursed,
    c.outstanding_interest,
    c.average_initial_price_usd,
    c.initial_collateral_value_usd,
    c.total_collateral_value_usd,
    c.last_btc_price_usd,

    p.payment_recorded_at_date_key,
    p.payment_recorded_at,
    p.payment_recorded_in_ledger_at_date_key,
    p.payment_recorded_in_ledger_at,
    p.disbursal_amount,
    p.interest_amount,

    ((c.total_collateral_value_usd / nullif(cfe.facility / 100.0, 0)) * 100.0) as facility_cvl,
    ((c.initial_collateral_value_usd / nullif(cfe.facility / 100.0, 0)) * 100.0) as initial_facility_cvl,

    ((c.total_collateral_value_usd / nullif(total_disbursed_amount / 100.0, 0)) * 100.0) as disbursed_cvl,
    ((cfe.terms_margin_call_cvl / nullif(cfe.facility, 0) * c.total_collateral) * 100000000.0 / (100.0 * 100.0)) as facility_margin_call_price_usd,
    ((cfe.terms_margin_call_cvl * d.total_disbursed_amount) / nullif(c.total_collateral, 0) * 100000000.0 / (100.0 * 100.0)) as disbursed_margin_call_price_usd,
    (((cfe.terms_liquidation_cvl * cfe.facility) / nullif(c.total_collateral, 0)) * 100000000.0 / (100.0 * 100.0)) as facility_liquidation_price_usd,

    (((cfe.terms_liquidation_cvl * d.total_disbursed_amount) / nullif(c.total_collateral, 0)) * 100000000.0 / (100.0 * 100.0)) as disbursed_liquidation_price_usd
from credit_facilities as cfe
full join int_cf_disbursals as d on cfe.event_id = d.event_id
full join int_cf_collaterals as c on cfe.event_id = c.event_id
full join int_cf_payments as p on cfe.event_id = p.event_id
