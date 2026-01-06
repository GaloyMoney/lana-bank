{{
    config(
        unique_key=["id", "version"],
    )
}}

with
    raw_stg_core_obligation_events_rollup as (
        select
            id,
            version,
            created_at,
            modified_at,
            amount,
            credit_facility_id,
            defaulted_account_id,
            defaulted_amount,
            defaulted_date,
            due_accounts,
            due_amount,
            due_date,
            effective,
            liquidation_date,
            not_yet_due_accounts,
            payment_allocation_amount,
            obligation_type,
            overdue_accounts,
            overdue_amount,
            overdue_date,
            payment_id,
            reference,
            ledger_tx_ids,
            payment_allocation_ids,
            is_completed,
            is_defaulted_recorded,
            is_due_recorded,
            is_overdue_recorded,
            liquidation_date__v_text,
            overdue_date__v_text,
            _dlt_load_id,
            _dlt_id
        from {{ source("lana", "core_obligation_events_rollup") }}
    )
select
    id as obligation_id,
    version,
    created_at,
    modified_at,
    amount,
    credit_facility_id,
    defaulted_account_id,
    defaulted_amount,
    defaulted_date,
    due_accounts,
    due_amount,
    due_date,
    effective,
    liquidation_date,
    not_yet_due_accounts,
    payment_allocation_amount,
    obligation_type,
    overdue_accounts,
    overdue_amount,
    overdue_date,
    payment_id,
    reference,
    ledger_tx_ids,
    payment_allocation_ids,
    is_completed,
    is_defaulted_recorded,
    is_due_recorded,
    is_overdue_recorded,
    liquidation_date__v_text,
    overdue_date__v_text,
    timestamp_micros(
        cast(cast(_dlt_load_id as decimal) * 1e6 as int64)
    ) as loaded_to_dw_at
from raw_stg_core_obligation_events_rollup
