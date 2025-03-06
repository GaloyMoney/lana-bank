{{ config(materialized='table') }}

with credit_facilities as (

    select * from {{ ref('int_credit_facilities') }}

),

int_cf_disbursals as (

    select * from {{ ref('int_cf_disbursals') }}

),

int_cf_collaterals as (

    select * from {{ ref('int_cf_collaterals') }}

),

int_cf_payments as (

    select * from {{ ref('int_cf_payments') }}

)

select
    cfe.*,

    d.idx,
    d.disbursal_concluded_event_recorded_at,
    d.amount,
    d.disbursal_concluded_event_recorded_at_date_key,

    c.audit_entry_id,
    c.recorded_in_ledger_at_date_key,
    c.recorded_in_ledger_at,
    c.action,
    c.collateralization_changed_event_recorded_at,
    c.collateralization_changed_state,
    c.total_collateral,
    c.price,
    c.collateralization_changed_event_recorded_at_date_key,
    c.diff,
    c.collateral,
    c.outstanding_disbursed,
    c.outstanding_interest,
    c.initial_collateral_value_usd,
    c.total_collateral_value_usd,
    c.last_btc_price_usd,

    p.disbursal_amount,
    p.interest_amount,

    d.recorded_at_date_key as disbursal_recorded_at_date_key,
    d.recorded_at as disbursal_recorded_at,

    d.event_type as disbursal_event_type,
    c.recorded_at_date_key as collateral_recorded_at_date_key,
    c.recorded_at as collateral_recorded_at,
    c.event_type as collateral_event_type,

    p.recorded_at_date_key as payment_recorded_at_date_key,
    p.recorded_at as payment_recorded_at,

    p.event_type as payment_event_type,
    p.recorded_in_ledger_at_date_key as payment_recorded_in_ledger_at_date_key,
    p.recorded_in_ledger_at as payment_recorded_in_ledger_at,
    ((c.total_collateral_value_usd / nullif(cfe.facility / 100.0, 0)) * 100.0) as facility_cvl,

    ((c.initial_collateral_value_usd / nullif(cfe.facility / 100.0, 0)) * 100.0) as initial_facility_cvl,
    ((c.total_collateral_value_usd / nullif(d.amount / 100.0, 0)) * 100.0) as disbursed_cvl,
    (((cfe.terms_margin_call_cvl * cfe.facility) / nullif(c.total_collateral, 0)) * 100000000.0 / (100.0 * 100.0)) as facility_margin_call_price_usd,
    (((cfe.terms_margin_call_cvl * d.amount) / nullif(c.total_collateral, 0)) * 100000000.0 / (100.0 * 100.0)) as disbursed_margin_call_price_usd,
    (((cfe.terms_liquidation_cvl * cfe.facility) / nullif(c.total_collateral, 0)) * 100000000.0 / (100.0 * 100.0)) as facility_liquidation_price_usd,
    (((cfe.terms_liquidation_cvl * d.amount) / nullif(c.total_collateral, 0)) * 100000000.0 / (100.0 * 100.0)) as disbursed_liquidation_price_usd
from credit_facilities as cfe
full join int_cf_disbursals as d on cfe.event_id = d.event_id
full join int_cf_collaterals as c on cfe.event_id = c.event_id
full join int_cf_payments as p on cfe.event_id = p.event_id
