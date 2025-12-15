{{ config(
    unique_key = ['id', 'version'],
) }}

with raw_stg_core_interest_accrual_cycle_events_rollup as (
    select
        id,
        version,
        created_at,
        modified_at,
        account_ids,
        accrued_at,
        amount,
        effective,
        facility_id,
        facility_maturity_date,
        idx,
        obligation_id,
        period,
        terms,
        total,
        tx_ref,
        ledger_tx_ids,
        is_interest_accruals_posted,
        _dlt_load_id,
        _dlt_id
    from {{ source("lana", "core_interest_accrual_cycle_events_rollup")}}
)
select
    id as interest_accrual_cycle_id,
    facility_id as credit_facility_id,
    version,
    created_at,
    modified_at,
    account_ids,
    accrued_at,
    amount,
    effective,
    facility_maturity_date,
    idx,
    obligation_id,
    period,
    terms,
    total,
    tx_ref,
    ledger_tx_ids,
    is_interest_accruals_posted,
    TIMESTAMP_MICROS(CAST(CAST(_dlt_load_id AS DECIMAL) * 1e6 as INT64 )) as loaded_to_dw_at
from raw_stg_core_interest_accrual_cycle_events_rollup
