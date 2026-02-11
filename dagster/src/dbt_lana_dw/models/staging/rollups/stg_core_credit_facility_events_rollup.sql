{{
    config(
        unique_key=["id", "version"],
    )
}}

with
    raw_stg_core_credit_facility_events_rollup as (
        select
            id,
            version,
            created_at,
            modified_at,
            account_ids,
            activated_at,
            amount,
            collateral,
            collateral_id,
            collateralization_ratio,
            collateralization_state,
            customer_id,
            customer_type,
            disbursal_credit_account_id,
            interest_accrual_cycle_idx,
            interest_period,
            maturity_date,
            outstanding,
            pending_credit_facility_id,
            price,
            public_id,
            structuring_fee_tx_id,
            terms,
            interest_accrual_ids,
            ledger_tx_ids,
            is_completed,
            is_matured,
            _dlt_load_id,
            _dlt_id
        from {{ source("lana", "core_credit_facility_events_rollup") }}
    )
select
    id as credit_facility_id,
    version,
    created_at,
    modified_at,
    account_ids,
    activated_at,
    amount,
    collateral,
    collateral_id,
    collateralization_ratio,
    collateralization_state,
    customer_id,
    customer_type,
    disbursal_credit_account_id,
    interest_accrual_cycle_idx,
    interest_period,
    maturity_date,
    outstanding,
    pending_credit_facility_id,
    price,
    public_id,
    structuring_fee_tx_id,
    terms,
    interest_accrual_ids,
    ledger_tx_ids,
    is_completed,
    is_matured,
    timestamp_micros(
        cast(cast(_dlt_load_id as decimal) * 1e6 as int64)
    ) as loaded_to_dw_at
from raw_stg_core_credit_facility_events_rollup
