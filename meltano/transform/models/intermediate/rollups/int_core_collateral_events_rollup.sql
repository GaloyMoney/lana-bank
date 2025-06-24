with credit_facility as (
    select
        id as collateral_id,
        credit_facility_id,

        action,
        abs_diff,
        collateral_amount,
        account_id,

        audit_entry_ids,
        ledger_tx_ids,

        * except(
            id,
            credit_facility_id,
            action,
            abs_diff,
            collateral_amount,
            account_id,
            last_sequence,
            created_at,
            modified_at,
            _sdc_received_at,
            _sdc_batched_at,
            _sdc_extracted_at,
            _sdc_deleted_at,
            _sdc_sequence,
            _sdc_table_version
        )
    from {{ ref('stg_core_collateral_events_rollup') }}
)


select * from credit_facility
