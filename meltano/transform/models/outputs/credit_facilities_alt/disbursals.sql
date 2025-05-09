{{ config(materialized='table') }}

select
    credit_facility_id,
    disbursal_id,
    recorded_at,
    disbursal_amount_usd,
    approved_disbursal_amount_usd,
    approved

from {{ ref('int_credit_facility_disbursals') }}
