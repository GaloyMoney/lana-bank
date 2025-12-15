{{ config(
    unique_key = ['id', 'version'],
) }}

with raw_stg_core_disbursal_events_rollup as (
    select
        id,
        version,
        created_at,
        modified_at,
        account_ids,
        amount,
        approval_process_id,
        approved,
        disbursal_credit_account_id,
        due_date,
        effective,
        facility_id,
        liquidation_date,
        obligation_id,
        overdue_date,
        public_id,
        ledger_tx_ids,
        is_approval_process_concluded,
        is_cancelled,
        is_settled,
        liquidation_date__v_text,
        overdue_date__v_text,
        _dlt_load_id,
        _dlt_id
    from {{ source("lana", "core_disbursal_events_rollup")}}
)
select
    id as disbursal_id,
    facility_id as credit_facility_id,
    version,
    created_at,
    modified_at,
    account_ids,
    amount,
    approval_process_id,
    approved,
    disbursal_credit_account_id,
    due_date,
    effective,
    liquidation_date,
    obligation_id,
    overdue_date,
    public_id,
    ledger_tx_ids,
    is_approval_process_concluded,
    is_cancelled,
    is_settled,
    liquidation_date__v_text,
    overdue_date__v_text,
    TIMESTAMP_MICROS(CAST(CAST(_dlt_load_id AS DECIMAL) * 1e6 as INT64 )) as loaded_to_dw_at
from raw_stg_core_disbursal_events_rollup
