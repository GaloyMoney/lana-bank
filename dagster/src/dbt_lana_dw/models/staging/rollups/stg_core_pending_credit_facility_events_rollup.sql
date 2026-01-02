{{
    config(
        unique_key=["id", "version"],
    )
}}

with
    raw_stg_core_pending_credit_facility_events_rollup as (
        select
            id,
            version,
            created_at,
            modified_at,
            account_ids,
            amount,
            approval_process_id,
            collateral,
            collateral_id,
            collateralization_ratio,
            collateralization_state,
            credit_facility_proposal_id,
            customer_id,
            customer_type,
            disbursal_credit_account_id,
            price,
            terms,
            ledger_tx_ids,
            is_collateralization_ratio_changed,
            is_collateralization_state_changed,
            is_completed,
            _dlt_load_id,
            _dlt_id,
            price__v_bigint
        from {{ source("lana", "core_pending_credit_facility_events_rollup") }}
    )
select
    id as pending_credit_facility_id,
    version,
    created_at,
    modified_at,
    account_ids,
    amount,
    approval_process_id,
    collateral,
    collateral_id,
    collateralization_ratio,
    collateralization_state,
    credit_facility_proposal_id,
    customer_id,
    customer_type,
    disbursal_credit_account_id,
    price,
    terms,
    ledger_tx_ids,
    is_collateralization_ratio_changed,
    is_collateralization_state_changed,
    is_completed,
    price__v_bigint,
    timestamp_micros(
        cast(cast(_dlt_load_id as decimal) * 1e6 as int64)
    ) as loaded_to_dw_at
from raw_stg_core_pending_credit_facility_events_rollup
