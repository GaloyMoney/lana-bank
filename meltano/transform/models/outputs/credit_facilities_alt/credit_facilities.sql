{{ config(materialized='table') }}

select
    id as credit_facility_id,
    recorded_at as created_at,
    json_value(parsed_event.customer_id) as customer_id,
    lax_int64(parsed_event.facility) / 100 as facility_usd,
    json_value(parsed_event.terms.accrual_interval.type) as terms_accrual_interval_type,
    lax_int64(parsed_event.terms.annual_rate) / 100 as terms_annual_rate,
    json_value(parsed_event.terms.duration.type) as terms_duration_type,
    lax_int64(parsed_event.terms.duration.value) as terms_duration_value,
    json_value(parsed_event.terms.incurrence_interval.type) as terms_incurrence_interval_type,
    lax_int64(parsed_event.terms.initial_cvl) / 100 as terms_initial_cvl,
    lax_int64(parsed_event.terms.liquidation_cvl) / 100 as terms_liquidation_cvl,
    lax_int64(parsed_event.terms.margin_call_cvl) / 100 as terms_margin_call_cvl,
    lax_int64(parsed_event.terms.one_time_fee_rate) / 100 as terms_one_time_fee_rate

from {{ ref('stg_credit_facility_events') }}

where event_type = "initialized"
