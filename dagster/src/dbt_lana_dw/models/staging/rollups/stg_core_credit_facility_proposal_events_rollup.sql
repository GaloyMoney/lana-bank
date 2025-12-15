{{ config(
    unique_key = ['id', 'version'],
) }}

with raw_stg_core_credit_facility_proposal_events_rollup as (
    select
        id,
        version,
        created_at,
        modified_at,
        amount,
        approval_process_id,
        custodian_id,
        customer_id,
        customer_type,
        disbursal_credit_account_id,
        status,
        terms,
        is_approval_process_concluded,
        _dlt_load_id,
        _dlt_id
    from {{ source("lana", "core_credit_facility_proposal_events_rollup")}}
)
select
    id as credit_facility_proposal_id,
    version,
    created_at,
    modified_at,
    amount,
    approval_process_id,
    custodian_id,
    customer_id,
    customer_type,
    disbursal_credit_account_id,
    status,
    terms,
    is_approval_process_concluded,
    TIMESTAMP_MICROS(CAST(CAST(_dlt_load_id AS DECIMAL) * 1e6 as INT64 )) as loaded_to_dw_at
from raw_stg_core_credit_facility_proposal_events_rollup
